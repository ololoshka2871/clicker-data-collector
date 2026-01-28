![clc](src/bin/clicker-data-collector-server/wwwroot/images/favicon.ico)
# Сборщик данных с устройства "Щелкунчик" 0x0000E007

## Устройство
- Репозиторий: [rk-meter (Щелкунчик)](https://bitbucket.org/Sctb_Elpa/rk-meter/src/master/)

## Назначение
Сбор данных с устройства "Щелкунчик" и экспорт их в форме Excel файлов.
Выполнено в форме Web-приложения.

## Windows 7 support
Используется [эта](https://doc.rust-lang.org/nightly/rustc/platform-support/win7-windows-msvc.html) инструкция.

### Сборка
Windows 7 - это специфический таргет для Rust, поэтому нужно вызывать сборку такой командой.
Потребуется установленная Visula Studuo и WindowsSDK в ней.
```shell
cargo +nightly run -Z build-std --release --target x86_64-win7-windows-msvc
```

Если команда давершается ошибкой вида:
`LINK : fatal error LNK1181: не удается открыть входной файл "windows.0.48.5.lib"`
1. Следует найти крейт `windows_x86_64_msvc` нужной версии в кеше `cargo`, лежащем по пути: `~\.cargo\registry\src\index.crates.io-...`.
2. Где-то там лежит нужный файлик. Затем указать путь до каталога с ним в переменой `WINDOWS_LIB_PATH` в файле `.cargo/link-fix.bat`
3. Перезапустить сборку
