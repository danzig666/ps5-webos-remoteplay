#!/usr/bin/env python3
import pathlib,json,subprocess
from source_snapshot import verify,NAME
root=pathlib.Path(__file__).resolve().parents[1]
for name,spec in json.loads((root/'upstream.lock.json').read_text()).items():
    if spec['role']=='reference only':continue
    path=root/'upstream'/name
    if (path/NAME).is_file():
        verify(path,spec)
        continue
    assert subprocess.check_output(['git','-C',str(path),'rev-parse','HEAD'],text=True).strip()==spec['rev'],name+' revision mismatch'
    assert not subprocess.check_output(['git','-C',str(path),'status','--porcelain'],text=True).strip(),name+' is dirty'
    for sub, rev in spec.get('submodules',{}).items():
        assert subprocess.check_output(['git','-C',str(path/sub),'rev-parse','HEAD'],text=True).strip()==rev
print('Pinned build input revisions verified')
