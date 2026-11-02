#!/usr/bin/env python3
"""Verify the extracted protocol exactly matches its pinned source plus reviewed patch."""
import pathlib,tempfile,subprocess,shutil
root=pathlib.Path(__file__).resolve().parents[1]
src=root/'upstream/ps5upload/engine/crates/ps5upload-core/src'
dst=root/'crates/psremote/src'
with tempfile.TemporaryDirectory(prefix='psremote-source-') as d:
    tmp=pathlib.Path(d)
    for p in dst.rglob('*'):
        if p.is_file() and p.name!='lib.rs':
            rel=p.relative_to(dst);(tmp/rel).parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src/rel,tmp/rel)
    subprocess.run(['patch','-p1','--batch','-i',str(root/'crates/psremote/webos.patch')],cwd=tmp,check=True,stdout=subprocess.DEVNULL)
    for p in tmp.rglob('*'):
        if p.is_file():assert p.read_bytes()==(dst/p.relative_to(tmp)).read_bytes(), 'unrecorded upstream modification: '+str(p)
print('PS5Upload extraction matches pinned source and patch')
