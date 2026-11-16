<div align="center">

# ForgeHelpYou

**Системные логи Linux — проще и понятнее.**

[![Download latest release](https://img.shields.io/github/v/release/abduforge/forgehelpyou?display_name=tag&style=for-the-badge&label=СКАЧАТЬ%20ДЛЯ%20LINUX)](https://github.com/abduforge/forgehelpyou/releases/latest)
[![CI](https://github.com/abduforge/forgehelpyou/actions/workflows/ci.yml/badge.svg)](https://github.com/abduforge/forgehelpyou/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Найди лог. Посмотри, что произошло. Найди ошибку. Без сложной настройки.**

[English documentation](README.md) · [Releases](https://github.com/abduforge/forgehelpyou/releases) · [Безопасность](SECURITY.md)

</div>

---

## Быстрый старт

1. Нажми **СКАЧАТЬ ДЛЯ LINUX** сверху и скачай архив для своей системы.
2. Распакуй архив.
3. Открой терминал в распакованной папке и выполни:

```bash
./install.sh
```

4. Посмотри, какие журналы доступны на компьютере:

```bash
forgehelpyou sources
```

5. Покажи последние системные сообщения:

```bash
forgehelpyou show syslog --lines 80
```

Готово. Если в твоём дистрибутиве нет `/var/log/syslog`, команда `forgehelpyou sources` покажет существующие журналы.

## Что умеет ForgeHelpYou

ForgeHelpYou — небольшая read-only CLI-утилита для просмотра и поиска по локальным журналам Linux.

| Что нужно сделать | Команда |
|---|---|
| Показать доступные журналы | `forgehelpyou sources` |
| Показать последние 100 строк | `forgehelpyou show syslog` |
| Показать последние 80 строк | `forgehelpyou show syslog --lines 80` |
| Найти строки со словом `failed` | `forgehelpyou show auth --contains failed --ignore-case` |
| Искать по всему журналу | `forgehelpyou search auth "authentication failure" --ignore-case` |
| Прочитать systemd journal | `forgehelpyou journal --since "1 hour ago" --priority warning` |
| Проверить окружение | `forgehelpyou doctor` |
| Открыть справку | `forgehelpyou --help` |

Вместо псевдонима (`syslog`, `auth`, `kern`, `dpkg` и другие) можно указать собственный путь к файлу.

## Какие журналы поддерживаются

Проверяются распространённые пути:

- `/var/log/syslog` и `/var/log/messages`
- `/var/log/auth.log` и `/var/log/secure`
- `/var/log/kern.log` и `/var/log/daemon.log`
- `/var/log/dpkg.log` и `/var/log/boot.log`
- `/var/log/Xorg.0.log`
- `/var/log/apt/history.log`

Если установлен `journalctl` и у пользователя есть права, можно читать журнал systemd. Файловые журналы работают независимо от systemd — это полезно для систем с SysVinit.

## Примеры поиска

Найти сообщения о неудачной аутентификации:

```bash
forgehelpyou search auth "failed" --ignore-case
```

Показать последние сообщения ядра:

```bash
forgehelpyou show kern --lines 120
```

Искать в указанном вручную файле:

```bash
forgehelpyou search ./my-app.log "connection refused" --ignore-case --limit 50
```

Попробовать на безопасном примере из проекта:

```bash
forgehelpyou search examples/sample-syslog.txt "failed" --ignore-case
```

Показать предупреждения systemd за последний час (только systemd):

```bash
forgehelpyou journal --since "1 hour ago" --priority warning
```

## Сборка из исходников

Требования: Linux и Rust 1.70 или новее.

```bash
cargo fmt --all
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
./install.sh
```

Проект использует только стандартную библиотеку Rust. Дополнительные Rust-крейты не нужны.

## Приватность и безопасность

- ForgeHelpYou читает журналы локально. Он не загружает, не изменяет, не ротирует и не удаляет их.
- Команды не запускаются через shell. Фильтры для journalctl передаются как отдельные аргументы.
- Программа не запускает `sudo` автоматически и не меняет права на файлы.
- Управляющие символы из обычных лог-файлов экранируются перед выводом в терминал.
- В логах могут быть имена пользователей, IP-адреса, имена компьютеров, данные сессий и иногда секреты. Перед публикацией отчётов проверь и удали чувствительные данные.
- Анализируй только свои системы или системы, которые тебе разрешено администрировать.

Если у пользователя нет доступа к журналу, используй учётную запись с соответствующими правами. Не делай системные логи доступными всем только ради этой программы.

## Удаление

```bash
./uninstall.sh
```

Команда удалит исполняемый файл из `~/.local/bin`. Системные журналы она не удаляет и не меняет.

## Состояние проекта

ForgeHelpYou 0.1.0 — ранняя версия. Сейчас проект умеет находить распространённые источники логов, показывать последние строки, искать текст в файлах и при наличии systemd читать его журнал. Новые функции будут добавляться после тестирования и обратной связи.

**Часть экосистемы AbduForge.**
