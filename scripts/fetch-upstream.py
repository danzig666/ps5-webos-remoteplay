#!/usr/bin/env python3
"""Fetch exact build inputs; reference repositories are optional."""
import json, pathlib, subprocess, sys
from source_snapshot import verify,NAME
root=pathlib.Path(__file__).resolve().parents[1]
for name, spec in json.loads((root/'upstream.lock.json').read_text()).items():
    if spec['role']=='reference only' and '--references' not in sys.argv: continue
    dest=root/'upstream'/name
    if (dest/NAME).is_file():
        verify(dest,spec)
        continue
    if not (dest/'.git').exists():
        dest.mkdir(parents=True,exist_ok=True)
        subprocess.run(['git','init',str(dest)],check=True)
        subprocess.run(['git','-C',str(dest),'remote','add','origin',spec['url']],check=True)
    head=subprocess.run(['git','-C',str(dest),'rev-parse','HEAD'],capture_output=True,text=True)
    if head.stdout.strip()!=spec['rev']:
        subprocess.run(['git','-C',str(dest),'fetch','--depth','1','origin',spec['rev']],check=True)
        subprocess.run(['git','-C',str(dest),'checkout','--detach',spec['rev']],check=True)
    if subprocess.check_output(['git','-C',str(dest),'status','--porcelain'],text=True).strip():
        sys.exit(f'{name}: dirty source; refusing build')

    for sub, rev in spec.get('submodules',{}).items():
        subprocess.run(['git','-C',str(dest),'submodule','update','--init',sub],check=True)
        assert subprocess.check_output(['git','-C',str(dest/sub),'rev-parse','HEAD'],text=True).strip()==rev
