# SQLite Practice в Rust

Этот проект демонстрирует работу с SQLite в Rust с использованием библиотеки `rusqlite`.

## Что реализовано

- Подключение к SQLite базе данных
- Создание таблицы, если она не существует
- CRUD операции (Create, Read, Update, Delete)
- Использование параметризованных запросов для безопасности
- Сериализация/десериализация с помощью `serde`
- Обработка ошибок с использованием `Result`
- Использование `Option<T>` для полей, которые могут быть null

## Структура проекта

- `Task` - модель данных для задачи
- `TaskDB` - менеджер базы данных с методами для работы с задачами

## Используемые библиотеки

- `rusqlite` - SQLite обертка для Rust
- `serde` - Сериализация/десериализация
- `tokio` - Асинхронность
- `anyhow` - Обработка ошибок

## Запуск проекта

```bash
cargo run
```

После запуска программа:
1. Создаст файл базы данных `tasks.db` (если он не существует)
2. Создаст таблицу `tasks` (если она не существует)
3. Добавит две задачи
4. Получит одну задачу по ID
5. Обновит одну задачу
6. Отметит задачу как выполненную
7. Выведет список всех задач
8. Удалит одну задачу
9. Выведет список оставшихся задач

## Задача для практики

Попробуйте расширить программу:

1. Добавьте возможность фильтрации задач (по статусу, дате)
2. Реализуйте интерактивный CLI интерфейс для управления задачами
3. Добавьте поддержку категорий для задач (новая таблица + связь many-to-many)
4. Реализуйте пагинацию при получении списка задач 