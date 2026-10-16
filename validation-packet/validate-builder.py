"""One-off native runtime validation. No Yard binary or assertion changes."""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import hashlib,json,os,re,shutil,subprocess,tempfile,threading,time

assert os.uname().sysname == 'Darwin'
assert subprocess.check_output(['id','-un'],text=True).strip() == 'runner'
packet=Path(__file__).resolve().parent
root=Path(tempfile.mkdtemp(prefix='yard-apple-fix-',dir='/private/tmp'))
out=root/'evidence';out.mkdir()
with open(os.environ['GITHUB_ENV'],'a') as f:f.write('YARD_APPLE_FIX_EVIDENCE='+str(out)+'\n')
env=os.environ.copy()
for key in ('NO_COLOR','BUILDKIT_COLORS'):env.pop(key,None)
cli='/opt/homebrew/bin/container'
dist=Path(os.environ['GITHUB_WORKSPACE'])/'shim-dist'
manifest=json.loads((dist/'manifest.json').read_text())
binary=dist/'container-builder-shim'
assert hashlib.sha256(binary.read_bytes()).hexdigest()==manifest['binary_sha256']
assert manifest['patch_sha256']=='77f48c21db678529c676726d4fdad46686b370e72196255525ae2b054fae75e1'
shutil.copytree(dist,out/'shim-identity')

def command(label,args,timeout=60,check=True):
    p=subprocess.run(args,env=env,stdin=subprocess.DEVNULL,capture_output=True,timeout=timeout)
    (out/(label+'.stdout')).write_bytes(p.stdout)
    (out/(label+'.stderr')).write_bytes(p.stderr)
    (out/(label+'.exit')).write_text(str(p.returncode)+'\n')
    if check and p.returncode:raise RuntimeError((label,p.returncode,p.stderr[-1000:]))
    return p

def snapshot(label):
    p=command(label,[cli,'list','--all','--format','json'])
    return json.loads(p.stdout)

before=snapshot('containers-before')
assert len([x for x in before if x['id']=='buildkit'])==1
unrelated={x['id']:x for x in before if x['id']!='buildkit'}
command('original-executables',[cli,'exec','buildkit','sha256sum','/usr/local/bin/container-builder-shim','/usr/bin/buildkitd'])
original_hash=(out/'original-executables.stdout').read_text().split()[0]
assert original_hash=='049a407f6bed33e5050988f6909b5c81787e6cd57856057a96cbc96b6d29b614'

# Retain the original same-key failure before changing the service.
p=command('original-control',['python3',str(packet/'reproduce-original.py'),'--equal-metadata'],timeout=240)
original=json.loads(p.stdout)
shutil.copytree(Path(original['root'])/'evidence',out/'original-control')
assert all(r.get('exit')==1 for r in original['results'][:2]),original
assert all(r.get('exit')==0 for r in original['results'][2:]),original
for i in (0,1):assert 'archive/tar: invalid tar header' in (out/'original-control'/str(i)).with_suffix('.log').read_text()

# Build a new image with the original builder image as its immutable base.
image='localhost/yard-y80-builder:transfer-private-'+manifest['binary_sha256'][:12]
context=root/'builder-context';context.mkdir()
shutil.copyfile(binary,context/'container-builder-shim')
(context/'Dockerfile').write_text('FROM ghcr.io/apple/container-builder-shim/builder@sha256:b5b3f7fa81e662db6929f1ad66d835d151a1b03f682cfe5f9fcb17fa46d6bcc9\nCOPY --chmod=0755 container-builder-shim /usr/local/bin/container-builder-shim\n')
command('build-candidate-image',[cli,'build','--file',str(context/'Dockerfile'),'--tag',image,str(context)],timeout=300)
command('candidate-image',[cli,'image','inspect',image])

# Change only build.image, preserving all other configuration bytes.
config_rows=[]
for path,default_mode in [
    (Path.home()/'.config/container/config.toml',0o600),
    (Path.home()/'Library/Application Support/com.apple.container/config/config.toml',0o444)]:
    assert not path.is_symlink(),str(path)
    data=path.read_text() if path.exists() else ''
    lines=data.splitlines(keepends=True)
    section=None;build_start=None;build_end=len(lines);image_line=None
    for n,line in enumerate(lines):
        header=re.match(r'^\s*\[([^\]]+)\]\s*(?:#.*)?$',line.strip())
        if header:
            if section=='build':build_end=n
            section=header.group(1).strip()
            if section=='build':
                assert build_start is None,'duplicate build section'
                build_start=n
        if section=='build' and re.match(r'^\s*image\s*=',line):
            assert image_line is None,'duplicate image'
            image_line=n
        assert not re.match(r'^\s*(?:build\s*=|build\.image\s*=)',line),'unsupported inline build configuration'
    mode=(path.stat().st_mode&0o777) if path.exists() else default_mode
    value='image = '+json.dumps(image)+'\n'
    if image_line is not None:lines[image_line]=value
    elif build_start is not None:lines.insert(build_end,value)
    else:lines.extend(['\n[build]\n',value])
    updated=''.join(lines)
    path.parent.mkdir(parents=True,exist_ok=True)
    fd,tmp=tempfile.mkstemp(prefix='.yard-build-image-',dir=str(path.parent))
    with os.fdopen(fd,'w') as f:f.write(updated)
    os.chmod(tmp,mode);os.replace(tmp,path)
    config_rows.append({'path':str(path),'before_sha256':hashlib.sha256(data.encode()).hexdigest(),'after_sha256':hashlib.sha256(updated.encode()).hexdigest(),'mode':oct(mode),'build_image':image})
(out/'config-change.json').write_text(json.dumps(config_rows,indent=2)+'\n')
command('stop-original-builder',[cli,'builder','stop'])
command('start-candidate-builder',[cli,'builder','start'],timeout=180)
after=snapshot('containers-after-switch')
for ident,row in unrelated.items():assert next(x for x in after if x['id']==ident)==row,ident
builder=next(x for x in after if x['id']=='buildkit')
assert builder['configuration']['image']['reference']==image,builder
command('candidate-executables',[cli,'exec','buildkit','sha256sum','/usr/local/bin/container-builder-shim','/usr/bin/buildkitd'])
assert (out/'candidate-executables.stdout').read_text().split()[0]==manifest['binary_sha256']
command('running-executable',[cli,'exec','buildkit','sh','-c','sha256sum /proc/1/exe; /usr/local/bin/container-builder-shim --version'])
assert (out/'running-executable.stdout').read_text().split()[0]==manifest['binary_sha256']

source=Path('/private/tmp/yard-parallel-context.fZ4owi/contexts')
rows=[];stamp=int(time.time())-60
for i in range(8):
    ctx=root/'candidate-contexts'/str(i)
    shutil.copytree(source/str(i),ctx,copy_function=shutil.copyfile)
    payload=ctx/'payload'
    for p in [*payload.rglob('*'),payload]:os.utime(p,(stamp,stamp))
    # Equal payload metadata with distinct Dockerfiles detects metadata cross-talk.
    with (ctx/'.yard/Dockerfile').open('a') as f:f.write('\nRUN printf '+str(i)+' > /yard-variant\n')
    expected={str(p.relative_to(payload)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(payload.rglob('*')) if p.is_file()}
    meta=[(str(p.relative_to(payload)),int(p.stat().st_birthtime),int(p.stat().st_mtime),p.stat().st_mode,p.stat().st_size) for p in sorted([*payload.rglob('*'),payload])]
    rows.append({'case':i,'context':str(ctx),'tag':'yard-y80-fixed-'+str(os.getpid())+'-'+str(i),'expected':expected,'metadata':meta})
assert rows[0]['expected']==rows[1]['expected']
assert rows[0]['metadata']==rows[1]['metadata']
(out/'candidate-inputs.json').write_text(json.dumps(rows,indent=2)+'\n')

def build_batch(rows,label):
    barrier=threading.Barrier(len(rows))
    def build(row):
        barrier.wait(timeout=30);start=time.monotonic()
        p=command(label+'-'+str(row['case']),[cli,'build','--file',row['context']+'/.yard/Dockerfile','--tag',row['tag'],row['context']],timeout=180,check=False)
        return {'case':row['case'],'exit':p.returncode,'seconds':time.monotonic()-start,'tag':row['tag']}
    with ThreadPoolExecutor(max_workers=len(rows)) as pool:result=list(pool.map(build,rows))
    (out/(label+'-results.json')).write_text(json.dumps(result,indent=2)+'\n')
    assert all(x['exit']==0 for x in result),result
    return result

results=build_batch(rows,'candidate')
for row in rows:
    i=row['case']
    p=command('payload-'+str(i),[cli,'run','--rm','--entrypoint','/bin/sh',row['tag'],'-c',"cd /probe && find . -type f -exec sha256sum {} \\; && printf 'VARIANT=' && cat /yard-variant"],timeout=90)
    observed={};variant=None
    for line in p.stdout.decode().splitlines():
        if line.startswith('VARIANT='):variant=int(line.split('=',1)[1])
        else:
            sha,name=line.split(None,1);observed[name.removeprefix('./')]=sha
    assert variant==i,(i,variant)
    assert observed==row['expected'],i

# A failed Dockerfile must not poison a later transfer of the same payload.
failed=root/'failed-context';shutil.copytree(Path(rows[0]['context']),failed,copy_function=shutil.copy2)
(failed/'.yard/Dockerfile').write_text('FROM pinfold/profile-default\nCOPY payload/ /probe/\nRUN exit 23\n')
failed_tag='yard-y80-fixed-failure-'+str(os.getpid())
p=command('intentional-build-failure',[cli,'build','--file',str(failed/'.yard/Dockerfile'),'--tag',failed_tag,str(failed)],timeout=180,check=False)
assert p.returncode!=0
assert 'exit code: 23' in (p.stdout+p.stderr).decode(),(p.stdout+p.stderr)[-2000:]
# A fresh invocation with unchanged payload metadata and a valid Dockerfile.
recovery=[dict(rows[0],case=80,tag='yard-y80-fixed-recovery-'+str(os.getpid()))]
recovery_results=build_batch(recovery,'after-failure')
p=command('recovery-payload',[cli,'run','--rm','--entrypoint','/bin/sh',recovery[0]['tag'],'-c',"cd /probe && find . -type f -exec sha256sum {} \\;"],timeout=90)
assert {line.split(None,1)[1].removeprefix('./'):line.split(None,1)[0] for line in p.stdout.decode().splitlines()}==rows[0]['expected']
command('remaining-private-staging',[cli,'exec','buildkit','sh','-c',"find /var/lib/container-builder-shim/fssync -maxdepth 1 -name 'transfer-*' -print"])
assert not (out/'remaining-private-staging.stdout').read_text().strip()
for row in rows+recovery:
    command('retire-'+str(row['case']),[cli,'image','delete',row['tag']])
final=snapshot('containers-final')
assert {x['id']:x for x in final if x['id']!='buildkit'}==unrelated
report={'status':'native candidate validation passed','root':str(root),'image':image,'builder':builder,'manifest':manifest,'original_results':original['results'],'candidate_results':results,'recovery_results':recovery_results,'payload_hashes_verified':sum(len(x['expected']) for x in rows),'candidate_layer_cache':'default','concurrency':8,'limits':['No forced mid-transfer cancellation observed in this script. Intentional failure occurs after COPY.','No repeated-transfer overhead benchmark yet.','Yard exact-source full qualification still required.']}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
