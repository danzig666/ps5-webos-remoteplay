#!/usr/bin/env python3
import importlib.util,pathlib,subprocess,tarfile,io,json
root=pathlib.Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('canonical',root/'scripts/canonical-ipk.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
p=root/'dist/org.homebrew.ps5remote.m0_0.1.0_arm.ipk'
assert mod.canonical(p.read_bytes())==p.read_bytes(),'IPK not canonical'
data=subprocess.check_output(['ar','p',str(p),'data.tar.gz'])
with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as t:
    members=t.getmembers()
    binaries=[m for m in members if m.name.endswith('/m0')]
    assert len(binaries)==1
    prefix=binaries[0].name[:-2]
    for rel in ['m0','lib/ss4s-ndl-webos5.so','lib/libSDL2-2.0.so.0','lib/libSDL2-webos.so.0','lib/libSDL3.so.0','assets/test.h264','appinfo.json','licenses/SDL.txt','LICENSE']:
        packed=t.extractfile(prefix+rel).read()
        assert packed==(root/'build/stage'/rel).read_bytes(),'IPK differs: '+rel
print('IPK canonical metadata and staged payload verified')
