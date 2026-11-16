// node scripts/util.check.ts
import { streak, daysLabel, ALL_DAYS, WEEKDAYS, parseLrc, lrcIndex, sameApp } from '../src/lib/util.ts';
import assert from 'node:assert';

const sun = new Date(2026, 10, 15); // domingo
const log = new Set(['2026-11-13', '2026-11-14']);
assert.equal(streak(log, ALL_DAYS, sun), 2, 'hoje pendente não quebra');
log.add('2026-11-15');
assert.equal(streak(log, ALL_DAYS, sun), 3, 'hoje feito conta');
assert.equal(streak(new Set(['2026-11-13']), WEEKDAYS, sun), 1, 'fim de semana é pulado');
assert.equal(streak(new Set(['2026-11-12']), ALL_DAYS, sun), 0, 'buraco quebra');
assert.equal(daysLabel(WEEKDAYS), 'dias úteis');
const l = parseLrc(['[ar:X]', '[00:12.50] Primeira', '[00:05.00]Intro', '[01:02.00][01:30.00] Refrão', 'sem tempo'].join(String.fromCharCode(10)));
assert.deepEqual(l.map(x => x.t), [5, 12.5, 62, 90], 'ordena e repete tags');
assert.equal(l[1].text, 'Primeira');
assert.equal(lrcIndex(l, 0), -1);
assert.equal(lrcIndex(l, 12.5), 1);
assert.equal(lrcIndex(l, 70), 2);
assert.equal(lrcIndex(l, 999), 3);
assert.ok(sameApp('Spotify.exe', 'Spotify'), 'spotify desktop');
assert.ok(sameApp('SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify', 'Spotify'), 'spotify da Microsoft Store');
assert.ok(sameApp('Chrome', 'chrome'), 'chrome');
assert.ok(sameApp('MSEdge', 'msedge'), 'edge');
assert.ok(sameApp('Brave', 'brave'), 'brave');
assert.ok(!sameApp('Spotify.exe', 'brave'), 'apps diferentes');
assert.ok(!sameApp('Spotify.exe', 'sp'), 'nome curto demais não casa');
console.log('util ok');
