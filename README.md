# Painel

Dashboard para um monitor secundário (feito para 1366×768): relógio, música tocando, volume, tarefas,
metas diárias, agenda e sensores do PC, em abas com layout editável.

Tauri 2 (Rust) + Svelte 5. Dados locais em SQLite (`%APPDATA%\app.painel.desktop\painel.db`).

## Instalar

Baixe o `Painel_<versão>_x64-setup.exe` em **[Releases](https://github.com/RootSpaceOrg/painel/releases/latest)** e rode.
Depois de instalado, o Painel avisa no topo da tela quando sai versão nova (e há o botão em ⚙ → Atualizações).

## Usar

- **Clique no título de um widget** (ou no relógio) para expandir em tela cheia; `Esc` fecha.
- **Abas**: dock embaixo; `←`/`→` ou `1`–`9` trocam.
- **Editar layout**: lápis no dock → arraste os cards, redimensione pelo canto, `+ widget`, `+ aba`, renomeie a aba no próprio dock.
- **Academia** (aba própria): registro do dia (peso, refeições, séries por grupo), semana (média de 7 dias, adesão, gráfico),
  treino (5–12 séries e 2–3× por grupo), plano (metas calculadas, descoberta da manutenção a 33 kcal/kg, revisão a cada 14 dias)
  e banco de alimentos (base TACO aproximada + busca no Open Food Facts) com refeições cadastradas.
- **Engrenagem**: monitor/tela cheia, wallpaper, escurecer/desfoque, agendas ICS, Spotify.
- Fechar a janela só esconde. Sair: ícone na bandeja.

### Agendas Google (várias contas)
Google Agenda → Configurações → a agenda → *Endereço secreto no formato iCal*. Cole um link por agenda.
Só leitura; eventos locais são criados no próprio painel.

### Spotify (opcional)
Sem configurar, o widget controla qualquer player do Windows (Spotify, YouTube no navegador…).
Para fila/playlists/curtir/dispositivos: crie um app em developer.spotify.com com redirect
`http://127.0.0.1:43821/callback`, cole o Client ID nas configurações e conecte.

## Desenvolver

Requisitos: Rust (rustup), Node, MSVC Build Tools, WebView2 (já vem no Windows 11).

```bash
cd app
npm i
npm run tauri dev     # app com hot reload
npm run tauri build   # instalador em src-tauri/target/release/bundle/nsis/
npm run check         # tipos da UI
node scripts/util.check.ts                # streak/recorrência
node scripts/fit.check.ts                 # cálculos da Academia
cd src-tauri && cargo test                # parser ICS
```

### Publicar uma versão

`main` recebe PR de `develop`. Para lançar: suba a versão em `app/package.json` (`npm version X.Y.Z --no-git-tag-version`)
no PR, faça o merge em `main` e, com `main` atualizada nesta máquina, rode `cd app && npm run release`: checks, build local
(com cache, ~1–2 min), tag e release no GitHub. Os Painéis instalados passam a oferecer a atualização.
Sem a máquina de build? Crie a tag e rode o workflow **Release** à mão no GitHub (mais lento, compila do zero).

- `src/lib/api.ts`: contrato UI ↔ Rust (todo comando/evento).
- `src/widgets/`: um arquivo por widget + `registry.ts`. Novo widget = componente com prop `expanded` + uma linha no registro.
- `src-tauri/src/`: um módulo por área (db, agenda, media, sys, audio, window, spotify).
