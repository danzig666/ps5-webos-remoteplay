#!/usr/bin/env python3
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1]
spec=json.loads((root/'tools.lock.json').read_text())[sys.argv[1]]
out=pathlib.Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
if not out.exists():subprocess.run(['curl','-fL',spec['url'],'-o',str(out)],check=True)
assert hashlib.file_digest(out.open('rb'),'sha256').hexdigest()==spec['sha256'],'archive checksum mismatch'
print('Verified',sys.argv[1],spec['sha256'])
