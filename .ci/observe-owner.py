#!/usr/bin/python3
"""Temporary CI observer. Real Pinfold, unchanged streams/status, no argv/env logs."""
import hashlib,json,os,pathlib,re,sqlite3,subprocess,sys,time
ROOT=pathlib.Path('/tmp/yard-ci-owner-observations')
ROOT.mkdir(mode=0o700,exist_ok=True)
def emit(kind,value):
    data=json.dumps({'at_ns':time.time_ns(),'kind':kind,**value},sort_keys=True)+'\n'
    fd=os.open(ROOT/'observations.jsonl',os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
    try:os.write(fd,data.encode())
    finally:os.close(fd)
def fixtures():return list(pathlib.Path('/tmp').glob('ye-*-g2-cleanup-inputs/p'))
def snapshot(project):
    keys=set();files=[]
    for f in project.glob('.yard/local/effects/*/lock'):
        try:
            s=f.stat();keys.add((s.st_dev,s.st_ino));files.append({'path':str(f),'device':s.st_dev,'inode':s.st_ino})
        except OSError:pass
    records=[];errors={'lock_table_errno':None,'process_directories_skipped':0,'descriptor_reads_skipped':0}
    try:
        for line in pathlib.Path('/proc/locks').read_text().splitlines():
            m=re.search(r'([0-9a-f]+):([0-9a-f]+):(\d+)',line)
            if not m:continue
            key=(os.makedev(int(m[1],16),int(m[2],16)),int(m[3]))
            if key in keys or ' POSIX ' in line:
                records.append(line);keys.add(key)
    except OSError as error:errors['lock_table_errno']=error.errno
    holders=[];scanned=0;truncated=False
    for proc in pathlib.Path('/proc').iterdir():
        if not proc.name.isdigit():continue
        try:
            for fd in (proc/'fd').iterdir():
                scanned+=1
                if scanned>20000:truncated=True;break
                try:
                    s=fd.stat()
                    if (s.st_dev,s.st_ino) not in keys:continue
                    stat=(proc/'stat').read_text().rsplit(') ',1)[1].split()
                    target=os.readlink(fd)
                    if not (target.startswith(str(project.parent)) or '/containers/storage/' in target):target='<outside scoped roots>'
                    holders.append({'pid':int(proc.name),'parent':int(stat[1]),'start_ticks':stat[19],'comm':(proc/'comm').read_text().strip(),'fd':int(fd.name),'device':s.st_dev,'inode':s.st_ino,'target':target,'wchan':(proc/'wchan').read_text().strip()})
                except (OSError,ValueError,IndexError):errors['descriptor_reads_skipped']+=1
        except OSError:errors['process_directories_skipped']+=1
        if truncated:break
    pressure={}
    for name in ('cpu','io','memory'):
        try:pressure[name]=pathlib.Path('/proc/pressure',name).read_text()
        except OSError:pass
    return {'project':str(project),'files':files,'locks':records,'descriptor_holders':holders,'descriptor_scan_truncated':truncated,'read_errors':errors,'pid_namespace':os.readlink('/proc/self/ns/pid'),'pressure':pressure}
if pathlib.Path(sys.argv[0]).name=='pinfold':
    args=sys.argv[1:];selected=[]
    if args[:2]==['box','list']:
        for p in fixtures():
            key=hashlib.sha256(os.fsencode(p)).hexdigest()[:10]
            if 'dev.yard.project='+key in args:selected.append(p)
    if not selected:os.execv('/usr/local/bin/pinfold',['pinfold',*args])
    started=time.monotonic()
    # Preserve inherited execution-lock descriptors. Never wrap streaming exec.
    import signal
    child=None;pending=[]
    def forward(signum,_frame):
        if child is None:pending.append(signum)
        else:
            try:child.send_signal(signum)
            except ProcessLookupError:pass
    for signum in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP):signal.signal(signum,forward)
    child=subprocess.Popen(['/usr/local/bin/pinfold',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=False)
    for signum in pending:forward(signum,None)
    stdout,stderr=child.communicate()
    for p in selected:
        try:
            key=hashlib.sha256(os.fsencode(p)).hexdigest()[:10]
            names=[];parse_errors=0;unscoped_names=0
            for line in stdout.splitlines():
                if not line.strip():continue
                try:
                    value=json.loads(line);name=value.get('name',value.get('box'))
                    if not isinstance(name,str):parse_errors+=1
                    elif re.fullmatch(r'yard-'+key+r'-[0-9]+',name):names.append(name)
                    else:unscoped_names+=1
                except (ValueError,TypeError,AttributeError):parse_errors+=1
            emit('inventory',{'project':str(p),'code':child.returncode,'returned_names':names,'inventory_parse_errors':parse_errors,'unscoped_names_count':unscoped_names,'seconds':time.monotonic()-started,'stdout_bytes':len(stdout),'stderr_bytes':len(stderr),'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'stderr_sha256':hashlib.sha256(stderr).hexdigest(),'runtime_timeout':b'runtime-timeout' in stderr or b'runtime-timeout' in stdout})
            if child.returncode:emit('inventory-failure-locks',snapshot(p))
        except Exception:pass
    sys.stdout.buffer.write(stdout);sys.stdout.buffer.flush();sys.stderr.buffer.write(stderr);sys.stderr.buffer.flush()
    if child.returncode<0:
        import signal
        
        if -child.returncode not in (signal.SIGKILL,signal.SIGSTOP):signal.signal(-child.returncode,signal.SIG_DFL)
        os.kill(os.getpid(),-child.returncode)
    sys.exit(child.returncode)
last={};deadline=time.monotonic()+1800
while not (ROOT/'done').exists() and time.monotonic()<deadline:
    for p in fixtures():
        try:
            c=sqlite3.connect(f'file:{p}/.yard/local/store.sqlite?mode=ro',uri=True,timeout=.05);c.row_factory=sqlite3.Row
            rows=[dict(r) for r in c.execute("SELECT id,kind,node,status,outcome,reserved,teardown,lock_contract FROM execution WHERE node='cleanup-audit-current' ORDER BY id")]
            items=[dict(r) for r in c.execute("SELECT id,kind,reason,execution,resolution FROM attention WHERE execution IN (SELECT id FROM execution WHERE node='cleanup-audit-current') ORDER BY id")];c.close()
            value={'project':str(p),'executions':rows,'attention':items};signature=json.dumps(value,sort_keys=True)
            if rows and last.get(str(p))!=signature:
                last[str(p)]=signature;emit('audit-state',value);emit('audit-state-locks',snapshot(p))
        except (OSError,sqlite3.Error):pass
    time.sleep(.1)
emit('observer-ended',{'done_marker':(ROOT/'done').exists()})
