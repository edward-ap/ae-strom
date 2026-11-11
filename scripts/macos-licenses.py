#!/usr/bin/env python3
"""Build a local, path-redacted attribution bundle for the locked macOS graph.
Python 3.9+, cargo and curl required. No crates are built or upstream code run.
Downloads only checksum-pinned registry archives and commit-pinned GitHub sources.
"""
import argparse, hashlib, io, json, os, pathlib, re, subprocess, tarfile, zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
PATTERN = re.compile(r'(?i)^(licen[cs]e|copying|copyright|notice|ofl)([._-]|$)')
def sha(data): return hashlib.sha256(data).hexdigest()
def is_notice(path):
    p = pathlib.PurePosixPath(path)
    return bool(PATTERN.match(p.name)) or any(x.lower() in ('licenses', 'licences') for x in p.parts[:-1])
def dump(path, value): path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
def fetch(url):
    return subprocess.check_output(['curl', '--fail', '--location', '--silent', '--show-error', '--retry', '2', '--max-time', '120', url])
def members(blob):
    with tarfile.open(fileobj=io.BytesIO(blob), mode='r:gz') as archive:
        return {m.name.partition('/')[2]: archive.extractfile(m).read() for m in archive.getmembers() if m.isfile() and '/' in m.name}
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', default='local-data/publication-0.1.4-macos-licenses')
    args = parser.parse_args()
    out = ROOT / args.output
    out.mkdir(parents=True, exist_ok=True)
    cargo = str(pathlib.Path.home()/'.cargo/bin/cargo')
    if not pathlib.Path(cargo).exists(): cargo = 'cargo'
    lock = (ROOT/'Cargo.lock').read_bytes()
    checksums = {}
    for section in lock.decode().split('[[package]]')[1:]:
        def field(k):
            m = re.search(r'^'+k+r' = "([^"]+)"', section, re.M)
            return m.group(1) if m else None
        checksums[(field('name'), field('version'))] = field('checksum')
    meta = json.loads(subprocess.check_output([cargo, 'metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', 'aarch64-apple-darwin'], cwd=ROOT))
    ids = {n['id'] for n in meta['resolve']['nodes']}
    packages = sorted((p for p in meta['packages'] if p['id'] in ids and p['source']), key=lambda p:(p['name'],p['version']))
    root = next(p for p in meta['packages'] if p['id'] == meta['resolve']['root'])
    inventory = {'schema':1, 'application':{'name':root['name'],'version':root['version']}, 'target':'aarch64-apple-darwin', 'scope':'Cargo resolved default-feature graph including build and dev dependencies, not proof of linked binary composition; excludes npm development tools and OS frameworks', 'cargo_lock_sha256':sha(lock), 'components':[], 'ui_files':[], 'limitations':[]}
    records = []
    def save(relative, data, origin, kind):
        parts = pathlib.PurePosixPath(relative)
        if parts.is_absolute() or '..' in parts.parts: raise RuntimeError('Unsafe archive path')
        path = out/relative; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(data)
        entry = {'file':relative, 'sha256':sha(data), 'bytes':len(data), 'source':origin, 'kind':kind}
        records.append(entry); return entry
    reference_url = 'https://github.com/edward-ap/ae-strom/releases/download/v0.1.3/ae-strom-0.1.3-licenses.zip'
    reference_sha = '288944d10cfb413ec07562fd3014dc225809df2b03378d0d2b59471245211cfd'
    reference_path = out/'windows-0.1.3-reference.zip'
    reference_blob = reference_path.read_bytes() if reference_path.exists() else fetch(reference_url)
    if sha(reference_blob) != reference_sha: raise RuntimeError('Reference release archive checksum mismatch')
    reference = zipfile.ZipFile(io.BytesIO(reference_blob))
    reference_inventory = json.loads(reference.read('LICENSE_INVENTORY.json'))
    inventory['fallback_reference'] = {'url':reference_url, 'sha256':reference_sha}
    inventory['generated_standard_fallbacks'] = []
    upstream = {}
    for p in packages:
        name, version = p['name'], p['version']; label = name+'-'+version
        url = 'https://static.crates.io/crates/{0}/{0}-{1}.crate'.format(name,version)
        manifest = pathlib.Path(p['manifest_path'])
        cache = manifest.parent.parent.parent.parent/'cache'/manifest.parent.parent.name/(label+'.crate')
        blob = cache.read_bytes() if cache.exists() else fetch(url)
        if sha(blob) != checksums[(name,version)]: raise RuntimeError('Registry checksum mismatch: '+label)
        files = members(blob)
        vcs = json.loads(files.get('.cargo_vcs_info.json', b'{}'))
        item = {'name':name, 'version':version, 'license_expression':p['license'], 'authors':p['authors'], 'repository':p['repository'], 'source_archive':url, 'source_archive_sha256':sha(blob), 'source_browser':'https://docs.rs/crate/{}/{}/source/'.format(name,version), 'vcs':vcs, 'license_files':[]}
        for f, data in sorted(files.items()):
            if is_notice(f) or f == p.get('license_file'):
                item['license_files'].append(save('crates/'+label+'/'+f, data, url+'#'+label+'/'+f, 'verbatim-registry-file'))
        if not item['license_files']:
            repo = (p['repository'] or '').removesuffix('.git') if hasattr(str, 'removesuffix') else (p['repository'] or '')
            commit = vcs.get('git',{}).get('sha1')
            if repo.startswith('https://github.com/') and commit and re.fullmatch('[0-9a-f]{40}',commit):
                archive_url = repo+'/archive/'+commit+'.tar.gz'
                if archive_url not in upstream:
                    archive_blob = fetch(archive_url)
                    upstream[archive_url] = (members(archive_blob), sha(archive_blob))
                upstream_files, archive_hash = upstream[archive_url]
                item['upstream_archive'] = {'url':archive_url, 'sha256':archive_hash}
                prefix = vcs.get('path_in_vcs','')
                ancestors = {'.'} | {str(x) for x in pathlib.PurePosixPath(prefix).parents} | {prefix}
                for f,data in sorted(upstream_files.items()):
                    path = pathlib.PurePosixPath(f)
                    if is_notice(f) and (str(path.parent) in ancestors or (prefix and f.startswith(prefix+'/')) or f.startswith('LICENSES/')):
                        source = repo+'/blob/'+commit+'/'+f
                        item['license_files'].append(save('crates/'+label+'/upstream/'+f,data,source,'verbatim-pinned-upstream-file'))
        if not item['license_files']:
            item['gap'] = 'No standalone license file found in published archive or relevant pinned upstream directories. Metadata declaration is not an upstream license text.'
            inventory['limitations'].append(label+': '+item['gap'])
            for f in ('Cargo.toml.orig','README.md','src/lib.rs','lib.rs'):
                if f in files:
                    save('crates/'+label+'/evidence/'+f,files[f],url+'#'+label+'/'+f,'verbatim-source-evidence-not-license')
        if not item['license_files'] and (name,version) in {('realfft','3.5.0'), ('selectors','0.38.0')}:
            prior = next(x for x in reference_inventory if x['name']==name and x['version']==version)
            if prior['license'] != p['license']: raise RuntimeError('Fallback license expression mismatch')
            prefix = 'rust/'+label+'/'
            evidence = ['Cargo.toml.orig','README.md'] + (['lib.rs'] if name=='selectors' else [])
            for f in evidence:
                if reference.read(prefix+f) != files[f]: raise RuntimeError('Fallback source evidence mismatch: '+label+'/'+f)
            standard = 'MIT-standard-license-template.txt' if name=='realfft' else 'MPL-2.0.txt'
            data = reference.read(prefix+standard)
            standard_source = prior['upstreamLicenseSources'][0]
            if sha(data) != standard_source['sha256']: raise RuntimeError('Standard license checksum mismatch')
            rec = save('crates/'+label+'/standard-reference/'+standard,data,standard_source['url'],'standard-license-reference-not-upstream-crate-file')
            rec['retrieved_from'] = reference_url+'#'+prefix+standard
            item['license_files'].append(rec)
            item['fallback_verification'] = {'same_version':True, 'same_license_expression':True, 'byte_identical_registry_evidence':evidence, 'reference_archive_sha256':reference_sha}
            item['evidence_files'] = ['crates/'+label+'/evidence/'+f for f in evidence]
            item['notice_status'] = 'Published declaration and preserved source evidence supplemented by labelled standard terms'
            if name=='realfft':
                item['license_files'].append(save('crates/'+label+'/standard-reference/ATTRIBUTION.txt',reference.read(prefix+'ATTRIBUTION.txt'),reference_url+'#'+prefix+'ATTRIBUTION.txt','prior-release-explanatory-attribution-not-upstream-notice'))
                item['review_caveat'] = 'Published MIT declaration and author attribution are preserved. The MIT template retains placeholders; it is not an upstream copyright notice and no copyright year or holder is inferred.'
            else:
                item['review_caveat'] = 'The source MPL notice is preserved with full standard MPL 2.0 text and exact registry source URL; no standalone upstream LICENSE is claimed.'
            inventory['limitations'].remove(label+': '+item.pop('gap'))
            inventory['generated_standard_fallbacks'].append({'component':label,'file':rec['file'],'kind':rec['kind'],'caveat':item['review_caveat']})
        inventory['components'].append(item)
        print(label, len(item['license_files']), flush=True)
    for f in ['LICENSE','web/vendor/LICENSE','web/vendor/THIRD_PARTY_NOTICES.md','web/vendor/fonts/NotoSans-OFL.txt','web/vendor/manifest.json','web/extensions/LICENSE']:
        inventory['ui_files'].append(save('ui/'+f,(ROOT/f).read_bytes(),'project:'+f,'verbatim-project-notice-or-provenance'))
    inventory['ui_runtime_hashes'] = [{'file':str(p.relative_to(ROOT)), 'sha256':sha(p.read_bytes())} for p in sorted((ROOT/'web').rglob('*')) if p.is_file() and p.suffix in ('.js','.css','.woff2')]
    inventory['component_count'] = len(packages)
    inventory['limitations'].append('Standard-reference supplements for realfft and selectors are explicitly labelled and are not upstream crate license files. realfft does not supply a standalone copyright notice; its standard MIT template retains placeholders without inventing ownership. Practical notice packaging is supplied, not legal assurance. System frameworks and non-shipped npm test/build tools are outside this Rust/UI inventory. No legal sufficiency conclusion is implied.')
    if (ROOT/'Cargo.lock').read_bytes() != lock: raise RuntimeError('Cargo.lock changed during inventory; rerun')
    dump(out/'LICENSE_INVENTORY.json',inventory)
    summary = '# macOS third-party license inventory\n\nApplication: '+root['name']+' '+root['version']+'\n\nTarget: aarch64-apple-darwin. '+str(len(packages))+' third-party Rust crates plus UI notices. Includes build/dev dependencies; not a binary linkage inventory.\n\nTwo explicitly labelled standard-reference supplements are used for realfft and selectors, validated against byte-identical published source declarations from the prior release. They are not upstream crate license files. Original notice bytes are preserved. Registry archives are checked against Cargo.lock; upstream fallback is pinned to .cargo_vcs_info.json revisions. All file hashes and public sources are in LICENSE_INVENTORY.json.\n\n## Remaining limitations\n\n'+'\n'.join('- '+s for s in inventory['limitations'])+'\n'
    (out/'README.md').write_text(summary)
    outputs = sorted({r['file'] for r in records} | {'LICENSE_INVENTORY.json','README.md'})
    (out/'SHA256SUMS').write_text(''.join(sha((out/f).read_bytes())+'  '+f+'\n' for f in outputs))
    archive = out/('ae-strom-'+root['version']+'-macos-licenses.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for f in outputs+['SHA256SUMS']:
            info = zipfile.ZipInfo(f,(2026,1,1,0,0,0)); info.compress_type = zipfile.ZIP_DEFLATED; info.external_attr = 0o100644 << 16
            z.writestr(info,(out/f).read_bytes())
    print(json.dumps({'count':len(packages),'inventory_sha256':sha((out/'LICENSE_INVENTORY.json').read_bytes()),'zip':archive.name,'zip_sha256':sha(archive.read_bytes()),'limitations':inventory['limitations']},indent=2))
if __name__ == '__main__': main()
