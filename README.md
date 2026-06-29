# rinha-de-backend-2026-rust

Bootstrap Rust para a Rinha de Backend 2026.

## Objetivo

Construir uma API de deteccao de fraude em transacoes de cartao usando busca vetorial, conforme a referencia oficial da competicao:

https://github.com/zanfranceschi/rinha-de-backend-2026

## Regras atuais a considerar

- A API final deve responder na porta `9999`.
- Os endpoints finais devem ser exatamente `GET /ready` e `POST /fraud-score`.
- A entrega final deve ser um `docker-compose.yml`.
- A arquitetura final deve ter pelo menos um load balancer e duas instancias da API em round-robin.
- O load balancer nao pode conter logica de deteccao.
- A soma dos limites de recursos dos servicos deve ficar em ate 1 CPU e 350 MB de memoria.
- A rede deve usar modo `bridge`; `host` e `privileged` nao sao permitidos.
- As imagens finais devem ser publicas e compativeis com `linux-amd64`.

## Proximos passos

- Escolher o framework HTTP Rust.
- Modelar o contrato de entrada e saida da API.
- Implementar `GET /ready`.
- Implementar a vetorizacao de 14 dimensoes conforme `REGRAS_DE_DETECCAO.md`.
- Carregar ou pre-processar os arquivos de referencia da competicao.
- Definir a topologia do `docker-compose.yml` com load balancer e duas APIs.

## Fora do bootstrap

Este repositorio ainda nao implementa endpoints, logica de negocio, workers, banco, cache, otimizacoes, benchmarks ou solucao da competicao.
