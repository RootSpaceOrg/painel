// npm run release — publica a versão de app/package.json a partir desta máquina.
// Build local reaproveita o cache de compilação (target/), bem mais rápido que o GitHub Actions,
// que recompilava tudo do zero a cada tag. Igual à Central: o instalador de Windows sai daqui.
import { execSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';

const sh = (cmd, opts = {}) => execSync(cmd, { stdio: 'inherit', ...opts });
const out = cmd => execSync(cmd).toString().trim();
const fail = msg => { console.error(`\n✖ ${msg}`); process.exit(1); };
const t0 = Date.now();

const version = JSON.parse(readFileSync('package.json', 'utf8')).version;
const tag = `v${version}`;

// só publica exatamente o que está em origin/main
sh('git fetch -q origin');
if (out('git status --porcelain')) fail('Há alterações não commitadas.');
if (out('git rev-parse HEAD') !== out('git rev-parse origin/main')) fail('Faça checkout de main atualizada (HEAD ≠ origin/main).');
if (out(`git ls-remote --tags origin ${tag}`)) fail(`A tag ${tag} já existe no GitHub. Suba a versão em package.json.`);

console.log(`\n▶ Checks`);
sh('npm run check');
sh('node scripts/util.check.ts');
sh('cargo test -q', { cwd: 'src-tauri' });

console.log(`\n▶ Build ${tag}`);
sh('npm run tauri build');
const exe = `src-tauri/target/release/bundle/nsis/Painel_${version}_x64-setup.exe`;
if (!existsSync(exe)) fail(`Instalador não encontrado: ${exe}`);

console.log(`\n▶ Publicando ${tag}`);
sh(`git tag ${tag}`);
sh(`git push -q origin ${tag}`);
sh(`gh release create ${tag} "${exe}" --title "Painel ${tag}" --generate-notes`);
console.log(`\n✔ ${tag} publicado em ${Math.round((Date.now() - t0) / 1000)}s`);
