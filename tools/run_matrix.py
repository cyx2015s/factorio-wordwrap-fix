"""Run native Factorio wrapping comparisons; never modifies the installation or user saves."""
import argparse, json, re, socket, subprocess, time
from pathlib import Path

TRACE=re.compile(r'TRACE font=(\S+) width=(-?\d+) height=(\d+) rich=(\w+) result=(\d+) overflow=(.*?) text=(.*?) lines=(.*)')

def debug_string(value):
    # Rust Debug uses \u{...} and \0; preserve ordinary escaped backslashes.
    def escape(m):
        if m[1]: return json.dumps(chr(int(m[1],16)))[1:-1]
        if m[2]: return r'\u0000'
        return m[0]
    return json.loads(re.sub(r'\\(?:u\{([0-9a-fA-F]+)\}|(0)|(.))',escape,value))

def records(text):
    rows=[]
    for line in text.splitlines():
        m=TRACE.search(line)
        if not m: continue
        font,w,h,rich,result,overflow,source,lines=m.groups()
        rows.append(dict(font=font,width=int(w),height=int(h),rich=rich=='true',result=int(result),overflow=json.loads(overflow),source=debug_string(source),lines=debug_string(lines)))
    return rows

def check(rows):
    errors=[]; normal_overflow=[]; narrow_overflow=[]
    norm=lambda s:re.sub(r'[ \t\r\n]','',s)
    for r in rows:
        src=norm(r['source']); dst=norm(''.join(r['lines']))
        if r['result']==0 and src!=dst: errors.append(dict(kind='text_loss',record=r))
        if r['result']==1 and not src.startswith(dst):errors.append(dict(kind='invalid_height_prefix',record=r))
        if r['rich'] and r['result']==0:
            for atom in re.findall(r'\[(?:item|entity|fluid|img)=[^\]]+\]',r['source']):
                if not any(atom in line for line in r['lines']):errors.append(dict(kind='split_atom',atom=atom,record=r))
        if r['overflow']:
            # Stress cases deliberately have indivisible punctuation runs wider
            # than tiny controls. Do not hide other width failures.
            forced=all(re.fullmatch(r'(?:[^\s，。！？：；、）】”]*[，。！？：；、）】”]+|[（【“]+[^\s])',r['lines'][i]) for i in r['overflow'])
            (narrow_overflow if forced else normal_overflow).append(r)
    return dict(records=len(rows),finite_records=sum(r['width']<2**31-1 for r in rows),fonts=len({r['font'] for r in rows}),errors=errors,normal_overflow=normal_overflow,indivisible_punctuation_overflow=narrow_overflow,height_limited=sum(r['result']==1 for r in rows))

def main():
    p=argparse.ArgumentParser();p.add_argument('--pid',required=True);p.add_argument('--dll',type=Path,required=True);p.add_argument('--injector',type=Path,required=True);p.add_argument('--log',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--port',type=int,default=34210)
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    def inject(mode=None):
        cmd=[str(a.injector),'--pid',a.pid]
        if mode:cmd.append(mode)
        subprocess.run(cmd+[str(a.dll)],check=True)
    def game(command):
        with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as s:
            s.settimeout(15);s.sendto(json.dumps(dict(command=command)).encode(),('127.0.0.1',a.port));reply=json.loads(s.recv(65535))
            assert reply['ok'],reply
        # The acknowledgement precedes GUI layout in the same frame.
        time.sleep(0.6)
    summary={}
    inject('--restore')  # also establishes a clean baseline when reusing a PID
    inject('--observe')
    for phase in ('vanilla','fixed','restored','reapplied'):
        if phase in ('fixed','reapplied'):inject();inject()  # idempotence
        elif phase=='restored':inject('--restore');inject('--restore')
        offset=a.log.stat().st_size if a.log.exists() else 0
        game('stress');game('encyclopedia');game('combinator')
        with a.log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8')
        (a.output/f'{phase}.log').write_text(raw,encoding='utf-8')
        rows=records(raw); assert len(rows)>=150,(phase,len(rows))
        (a.output/f'{phase}.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2),encoding='utf-8')
        summary[phase]=check(rows)
        if phase in ('fixed','reapplied'):
            assert not summary[phase]['errors'],summary[phase]['errors'][:1]
            assert not summary[phase]['normal_overflow'],summary[phase]['normal_overflow'][:1]
    # Exact original outputs must return after restore (compare stable inputs,
    # font pointers and dimensions, independent of trace ordering).
    def keyed(phase):
        rows=json.loads((a.output/f'{phase}.json').read_text(encoding='utf-8'))
        return {(r['font'],r['width'],r['height'],r['rich'],r['source']):(r['result'],r['lines'],r['overflow']) for r in rows}
    for before,after in [('vanilla','restored'),('fixed','reapplied')]:
        x,y=keyed(before),keyed(after); common=x.keys()&y.keys()
        assert len(common)>=140,(before,after,len(common))
        assert all(x[k]==y[k] for k in common),(before,after,'output mismatch')
        summary[f'{before}_equals_{after}']=len(common)
    inject('--trace-off');game('encyclopedia')
    (a.output/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps({k:{n:v for n,v in d.items() if n in ('records','finite_records','fonts','height_limited')} if isinstance(d,dict) else d for k,d in summary.items()},indent=2))
if __name__=='__main__':main()
