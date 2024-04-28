# WireGuard Management Utility wgdhc

## Описание

Эта утилита предназначена для автоматического управления Wireguard. Есть выделенный сервер, который заведует адресами и автоматически добавляет пиры по запросу. Так же реализован клиент но в случае необходимости его можно полностью заменить, поскольку все взаимодействие идет через grpc описанный в `./proto`

## Возможности

- **Инициализация конфигурации**: Создает начальное хранилище из конфигурационного файла `~/.config/wgdhc.yaml`.
- **Запуск сервера**: Запускает сервер WireGuard на основе данных из хранилища и из конфигурационного файла `~/.config/wgdhc.yaml`.
- **Просмотр данныз**: Отображает сохраненные профили сервиса(запускается только на самом сервере).
- **Клиентская команда**: совершает запрос к серверу и инициализирует интерфейс wg.

## Установка
Cборка проекта производится с помощью `cargo` с toolchain nightly.
шаги:
 - [установить rust](https://www.rust-lang.org/tools/install)
 - установить nightly с помощью `rustup override set nightly` в папке проекта
 - собрать коммандой `cargo build --release`
 - в `./target/release/` будет лежать исполняемый файл `wgdhc`

## Использование
утилита предназначена только для работы на linux

для корректной работы сервера требуются дополнительные настроки системы (sysctl)
```
net.ipv4.ip_forward=1
net.ipv6.conf.all.forwarding=1
```
после чего необходимо создать конфигурационный файл `~/.config/wgdhc.yaml`


для корректной работы клиента требуется только wg
```
wgdhc client 'http://service_ip:port' <account>
```

конкретные команды и их аргументы можно посмотреть через `--help`

удобный способ попробовать - docker контейнеры,
все необходимое лежит в `docker`, демонстрацию можно запустить оттуда командой `docker-compose up` и подключиться к двум контенерам-клиентам через `docker exec -it <container> bash`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### MIT License Summary

The MIT License is a permissive license that is short and to the point. It lets people do anything they want with your code as long as they provide attribution back to you and don’t hold you liable.

