#!/usr/bin/env python3
import pathlib,subprocess,tarfile,io,importlib.util,json
root=pathlib.Path(__file__).resolve().parents[1]
p=root/'dist/org.homebrew.ps5remote_0.2.0_arm.ipk'
spec=importlib.util.spec_from_file_location('canonical',root/'scripts/canonical-ipk.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
assert p.read_bytes()==mod.canonical(p.read_bytes())
data=subprocess.check_output(['ar','p',str(p),'data.tar.gz'])
with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as t:
    bins=[m.name for m in t.getmembers() if m.name.endswith('/ps5remote') and m.isfile()]
    assert len(bins)==1
    prefix=bins[0][:-len('ps5remote')]
    for f in (root/'build/client-stage').rglob('*'):
        if f.is_file():assert t.extractfile(prefix+str(f.relative_to(root/'build/client-stage'))).read()==f.read_bytes()
    app=json.load(t.extractfile(prefix+'appinfo.json'));assert app['main']=='ps5remote' and app['id']=='org.homebrew.ps5remote'
    assert not any(m.name.endswith(('/pairing.json','/settings.json')) for m in t.getmembers())
print('Native client IPK metadata, complete payload and absence of user configuration verified')
