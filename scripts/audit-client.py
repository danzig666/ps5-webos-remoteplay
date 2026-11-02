#!/usr/bin/env python3
import pathlib,subprocess,json,re,hashlib
root=pathlib.Path(__file__).resolve().parents[1];stage=root/'build/client-stage'
for name in ['ps5remote','lib/ss4s-ndl-webos5.so','lib/libSDL2-2.0.so.0','lib/libSDL2-webos.so.0','lib/libSDL3.so.0','assets/gamecontrollerdb.txt','appinfo.json','LICENSE']:
    assert (stage/name).is_file(),'missing '+name
assert json.loads((stage/'appinfo.json').read_text())['id']=='org.homebrew.ps5remote'
allowed={'libc.so.6','libpthread.so.0','libdl.so.2','librt.so.1','libm.so.6','ld-linux.so.3','libNDL_directmedia.so.1'}
for p in stage.rglob('*'):
    if not p.is_file():continue
    b=p.read_bytes()
    if b[:4]==b'\x7fELF':
        assert b[4]==1 and int.from_bytes(b[18:20],'little')==40 and not(int.from_bytes(b[36:40],'little')&0x400),'wrong ARM ABI'
        desc=subprocess.check_output(['readelf','-d',str(p)],text=True)
        for dep in re.findall(r'NEEDED.*?\[(.*?)\]',desc):assert dep in allowed or (stage/'lib'/dep).is_file(),'unpackaged dependency '+dep
        print(hashlib.sha256(b).hexdigest(),p.relative_to(stage))
assert b'SIMULATED_NATIVE' not in (stage/'ps5remote').read_bytes(), 'test double must not ship'
assert (stage/'lib/libSDL2-2.0.so.0').stat().st_size<100000,'SDK dummy SDL must not be shipped'
print('Client ELF/assets/dependency audit passed; TV/runtime functionality UNVERIFIED')
