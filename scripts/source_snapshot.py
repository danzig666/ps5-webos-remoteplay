"""Validate portable, hash-manifested source snapshots made from pinned Git trees."""
import pathlib,json,hashlib
NAME='.source-snapshot.json'
def verify(path,spec):
    path=pathlib.Path(path);meta=json.loads((path/NAME).read_text())
    assert meta['revision']==spec['rev'] and meta['url']==spec['url'],'snapshot revision/source mismatch'
    assert meta.get('submodules',{})==spec.get('submodules',{}),'snapshot gitlink mismatch'
    for rel,digest in meta['files'].items():
        p=path/rel
        assert not pathlib.PurePosixPath(rel).is_absolute() and '..' not in pathlib.PurePosixPath(rel).parts
        assert p.is_file() and not p.is_symlink(),'missing/unsafe source '+rel
        assert hashlib.sha256(p.read_bytes()).hexdigest()==digest,'source snapshot modified: '+rel
    actual={str(p.relative_to(path)) for p in path.rglob('*') if p.is_file() and p.name!=NAME}
    assert actual==set(meta['files']),'unexpected files in source snapshot'
