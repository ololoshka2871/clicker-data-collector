$.noty.defaults.theme = "bootstrapTheme"

function noty_error(text: string, timeot: number = 5000): Noty {
    return noty({
        type: "error",
        text: "<i class='fa fa-times'></i> Ошибка: " + text,
        timeout: timeot
    });
}

function noty_success(text: string = "Операция выполнена успешно.", timeout: number = 3000): Noty {
    return noty({
        type: "success",
        text: "<i class='fa fa-check'></i> " + text,
        timeout: timeout
    });
}

function round_to_2_digits(x?: number): string {
    if (x === undefined) {
        return "0"
    } else {
        const s = (Math.round(x * 100) / 100).toString();
        const dot_index = s.indexOf('.');
        if (dot_index < 0) {
            return s + '.00';
        } else if (dot_index == s.length - 2) {
            return s + '0'
        } else {
            return s
        }
    }
}

function patch_value(target: HTMLInputElement, field_name: string) {
    const val = $(target).val() || "0.0";

    const new_value = parseFloat(val as string);

    if (typeof (new_value) !== 'number') {
        return;
    }

    let v = {}
    v[field_name] = new_value;

    $.ajax({
        url: '/config',
        method: 'PATCH',
        data: JSON.stringify(v),
        contentType: 'application/json',
        error: (e) => {
            noty_error(e.responseText || e.statusText);
        }
    });
}