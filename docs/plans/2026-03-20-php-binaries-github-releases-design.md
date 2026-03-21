# Design: Distribuição de Binários PHP via GitHub Releases

## Introdução

Este documento define a abordagem para distribuir binários compilados do PHP para download automatizado, utilizando recursos nativos do GitHub.

## Objetivo

Permitir que os usuários façam download automatizado das versões do PHP (plataformas Linux, macOS, Windows) com versionamento público e URLs estáveis.

## Solução Recomendada

**GitHub Releases com CI automático via tags Git**.

Quando uma tag como `v8.5.4` for criada, o CI (GitHub Actions) compactará os binários por plataforma em arquivos `.tar.gz` e os anexará a uma release do GitHub.

## Arquitetura

```
Tag criada (v8.5.4)
       ↓
GitHub Actions (workflow: release.yml)
       ↓
Compactar binários por plataforma
       ↓
Criar release com assets
       ↓
URLs de download disponíveis
```

## Componentes

### 1. Workflow GitHub Actions (`release.yml`)
- **Trigger**: Push de tag (`v*`)
- **Jobs**: 
  - `package`: Compacta binários por plataforma
  - `release`: Cria release e anexa assets

### 2. Script de Embalagem (`scripts/package-release.sh`)
- Compacta cada diretório de plataforma em `.tar.gz`
- Gera checksums SHA256
- Organiza assets por plataforma

### 3. Estrutura de Assets
- `php-{version}-{platform}.tar.gz` (ex: `php-8.5.4-linux-x64.tar.gz`)
- `checksums-{version}.txt` (lista de checksums)

## Fluxo de Trabalho

### Para o Maintainer (criar release):
1. Compilar binários (já existe no repositório)
2. Criar tag Git: `git tag v8.5.4`
3. Push da tag: `git push origin v8.5.4`
4. CI cria release automaticamente

### Para o Usuário (download):
1. Consultar última release via API: `GET /repos/{owner}/{repo}/releases/latest`
2. Encontrar asset pela plataforma
3. Download direto via URL do asset

## Implementação

### 1. Criar workflow `release.yml`
```yaml
name: Release PHP Binaries
on:
  push:
    tags:
      - 'v*'
jobs:
  package-and-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Package binaries
        run: ./scripts/package-release.sh
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*.tar.gz
            dist/*.txt
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 2. Criar script `scripts/package-release.sh`
- Extrair versão da tag
- Para cada plataforma em builds/:
  - Compactar em `dist/php-{version}-{platform}.tar.gz`
  - Calcular SHA256
- Gerar `dist/checksums-{version}.txt`

### 3. Atualizar `.gitignore`
- Adicionar `dist/` para não versionar os compactados

## Vantagens

- **Simples**: Usa recursos nativos do GitHub
- **Versionamento**: Tags Git = versões
- **Acesso público**: URLs públicas por padrão
- **Automatizado**: CI cria releases
- **Sem custos extras**: GitHub oferece armazenamento gratuito para releases (limite 50GB por repositório)
- **API do GitHub**: Permite download automatizado via API

## Limitações

- **Limite de tamanho**: 2GB por arquivo, 50GB total por repositório
- **Sem granularidade**: Não é possível fazer download de partes específicas
- **Custos de largura de banda**: GitHub pode limitar transferências em repositórios públicos (mas geralmente generoso)

## Próximos Passos

1. Criar script de embalagem
2. Criar workflow GitHub Actions
3. Testar com tag de desenvolvimento
4. Documentar uso da API para download
5. Atualizar README com instruções

## Referências

- [GitHub Releases API](https://docs.github.com/en/rest/releases)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
- [GitHub Docs: Managing releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)