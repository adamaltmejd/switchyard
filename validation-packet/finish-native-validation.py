"""Finish the retained native build trial with an externally held image reader."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile,time

assert os.uname().sysname=='Darwin'
root=Path('/private/tmp/yard-apple-fix-65qixixy')
old=root/'evidence'
out=Path(tempfile.mkdtemp(prefix='yard-apple-fix-finish-',dir='/private/tmp'))
with open(os.environ['GITHUB_ENV'],'a') as f:f.write('YARD_APPLE_FIX_EVIDENCE='+str(out)+'\n')
env=os.environ.copy()
for k in ('NO_COLOR','BUILDKIT_COLORS'):env.pop(k,None)
cli='/opt/homebrew/bin/container'
manifest=json.loads((old/'shim-identity/manifest.json').read_text())
rows=json.loads((old/'candidate-inputs.json').read_text())
results=json.loads((old/'candidate-results.json').read_text())
assert len(results)==8 and all(x['exit']==0 for x in results)
shutil.copytree(old,out/'first-trial')

def cmd(label,args,timeout=90,check=True):
    start=time.monotonic()
    p=subprocess.run(args,env=env,stdin=subprocess.DEVNULL,capture_output=True,timeout=timeout)
    (out/(label+'.stdout')).write_bytes(p.stdout);(out/(label+'.stderr')).write_bytes(p.stderr)
    (out/(label+'.json')).write_text(json.dumps({'command':args,'exit':p.returncode,'seconds':time.monotonic()-start})+'\n')
    if check and p.returncode:raise RuntimeError((label,p.returncode,p.stderr[-1000:]))
    return p

p=cmd('running-candidate',[cli,'exec','buildkit','sha256sum','/proc/1/exe','/usr/local/bin/container-builder-shim','/usr/bin/buildkitd'])
assert p.stdout.decode().split()[0]==manifest['binary_sha256']
assert p.stdout.decode().splitlines()[2].split()[0]=='2499f1a65136cfa6f4d3c7fb523cbe601376ecc46538d555f3f64abe769aa536'
before=json.loads(cmd('containers-before',[cli,'list','--all','--format','json']).stdout)
before_ids={x['id'] for x in before}

def inspect_payload(row,label,variant=True):
    # Holding the init process removes the unrelated run-vs-fast-exit race.
    name='yard-y80-payload-'+str(os.getpid())+'-'+str(row['case'])
    assert name not in before_ids
    cmd(label+'-start',[cli,'run','--detach','--name',name,'--entrypoint','/bin/sh',row['tag'],'-c','mkfifo /tmp/yard-inspection-hold; read -r held < /tmp/yard-inspection-hold'])
    try:
        script="cd /probe && find . -type f -exec sha256sum {} \\;"
        if variant:script+=" && printf 'VARIANT=' && cat /yard-variant"
        p=cmd(label+'-contents',[cli,'exec',name,'/bin/sh','-c',script])
        observed={};got=None
        for line in p.stdout.decode().splitlines():
            if line.startswith('VARIANT='):got=int(line.split('=',1)[1])
            else:
                sha,path=line.split(None,1);observed[path.removeprefix('./')]=sha
        assert observed==row['expected'],row['case']
        if variant:assert got==row['case'],(row['case'],got)
        return {'case':row['case'],'files':len(observed),'variant':got}
    finally:
        cmd(label+'-stop',[cli,'stop',name])
        cmd(label+'-delete',[cli,'delete',name])

verified=[inspect_payload(r,'verify-'+str(r['case'])) for r in rows]
# Same payload/metadata, differing Dockerfile. Fail after unpack, then rebuild it.
failed=root/'failure-context'
assert not failed.exists()
shutil.copytree(Path(rows[0]['context']),failed,copy_function=shutil.copy2)
def metadata(payload):
    return [(str(p.relative_to(payload)),int(p.stat().st_birthtime),int(p.stat().st_mtime),p.stat().st_mode,p.stat().st_size) for p in sorted([*payload.rglob('*'),payload])]
assert metadata(failed/'payload')==[tuple(x) for x in rows[0]['metadata']]
(failed/'.yard/Dockerfile').write_text('FROM pinfold/profile-default:latest\nCOPY payload/ /probe/\nRUN exit 23\n')
failed_tag='yard-y80-failed-'+str(os.getpid())
p=cmd('intentional-failure',[cli,'build','--file',str(failed/'.yard/Dockerfile'),'--tag',failed_tag,str(failed)],timeout=180,check=False)
assert p.returncode!=0
assert 'exit code: 23' in (p.stdout+p.stderr).decode(),(p.stdout+p.stderr)[-2000:]
(failed/'.yard/Dockerfile').write_text('FROM pinfold/profile-default:latest\nCOPY payload/ /probe/\nRUN printf repaired > /yard-recovery\n')
cmd('fresh-transfer-after-failure',[cli,'build','--file',str(failed/'.yard/Dockerfile'),'--tag',failed_tag,str(failed)],timeout=180)
recovered=inspect_payload(dict(rows[0],case=80,tag=failed_tag),'verify-recovery',variant=False)
p=cmd('remaining-transfer-staging',[cli,'exec','buildkit','sh','-c',"find /var/lib/container-builder-shim/fssync -maxdepth 1 -name 'transfer-*' -print"])
assert not p.stdout.strip()
after=json.loads(cmd('containers-after',[cli,'list','--all','--format','json']).stdout)
assert {x['id'] for x in after}==before_ids
for row in rows+[{'case':80,'tag':failed_tag}]:cmd('retire-image-'+str(row['case']),[cli,'image','delete',row['tag']])
report={'status':'native patch validation passed','patched_binary':manifest,'initial_trial':str(root),'original_negative_control':json.loads((old/'original-control.stdout').read_text()),'candidate_parallel_results':results,'payload_verification':verified,'after_failed_build':recovered,'cleanup':'No transfer-owned staging remained; only named owned image readers/tags retired. Builder retained patched.','separate_original_inspection_failure':{'case':6,'exit':1,'reason':'no runtime client exists: container is stopped','output':'Printed expected marker hash and VARIANT=6; retained as failed, not accepted as passing.'},'limits':['Failure occurs after COPY; mid-transfer cancellation not yet tested.','Repeated receiver transfer cost not yet benchmarked.','Full Yard qualification is a separate job.']}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
