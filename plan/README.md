# План переноса Z на Rust

Этот каталог - рабочий источник правды перед любыми новыми изменениями.

Перед каждым срезом:

1. Прочитать `plan/README.md`, `plan/porting-log.md`, `plan/dead-code-audit.md` и `plan/engine-port-strategy.md`.
2. Выбрать один проверяемый срез из C/C++ source.
3. Зафиксировать в плане, какой оригинальный файл/функция является источником правды.
4. Переносить только после сравнения с оригиналом, без "примерно похоже".
5. После изменения обновить `plan/porting-log.md` и статус проверки.

Текущее правило по ownership:

- Concrete-unit файлы лежат как `src/units/[type]/[unit_name]/[unit_name]_logic.rs`, `src/units/[type]/[unit_name]/[unit_name]_ui.rs`, `src/units/[type]/[unit_name]/[unit_name]_mod.rs`.
- Визуальное отображение конкретных юнитов лежит в `[unit_name]_ui.rs` или family UI facade.
- Числа, настройки, combat/behavior policy конкретного юнита лежат в `[unit_name]_logic.rs`, а не в `main.rs`, `placement.rs`, `components.rs`.
- `[unit_name]_mod.rs` связывает logic/ui и сохраняет family facade compatibility.
- Bevy orchestration, queries, spawning и wiring могут оставаться в верхних системах, но доменная логика должна быть вынесена в `src/units`.

Проверки перед закрытием среза:

- `cargo check -q`
- `cargo test -q`
- `./scripts/build-wasm.sh`
- smoke в браузере: canvas не черный, boot state started
