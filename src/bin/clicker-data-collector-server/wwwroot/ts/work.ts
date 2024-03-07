// define interface for jquery JQuery<HTMLElement> where method tooltip is defined
interface JQuery<TElement extends Element = HTMLElement> extends Iterable<TElement> {
    tooltip(options?: any): JQuery<TElement>;
}

// declare hotkeys as defined global function
declare function hotkeys(key: string, callback: (event: KeyboardEvent, handler: any) => void): void;

// ---------------------------------------------------------------------------------------------

interface IResonatorData {
    id: number,
    timestamp: string,
    F: number,
    F_deviation: number,
    Rk: number,
    Rk_deviation: number,
    comment: String,
}

interface IBoxPlot {
    median: number,
    q1: number,
    q3: number,
    iqr: number,
    lower_bound: number,
    upper_bound: number,
}

interface IMeasureProcessStat {
    timestamp: number,
    state: string,

    freqs: Array<number>,
    rks: Array<number>,

    freqs_avg?: IBoxPlot,
    rks_avg?: IBoxPlot,
}

// ---------------------------------------------------------------------------------------------

const schema = joi.object({
    data_type: joi.string().min(1).required(),
    route_id: joi.string().min(1).required(),
    ambient_temperature_range: joi.string().min(1).required(),
    date: joi.date().required(),
});

let present_noty: Noty = null;
let grid: Types.Grid<any, any> = null;
let MPdailog: Types.Dialog = null;

// on page loaded jquery
$(() => {
    // https://www.chartjs.org/docs/2.9.4/getting-started/integration.html#content-security-policy
    Chart.platform.disableCSSInjection = true;

    // https://getbootstrap.com/docs/4.0/components/tooltips/
    $('[data-toggle="tooltip"]').tooltip()

    // ref: 
    // 1. https://gijgo.com/grid/demos/jquery-grid-material-design : New record
    // 2. https://gijgo.com/grid/demos/bootstrap-grid-inline-edit : Inline edit
    // 3. https://gijgo.com/grid/demos/bootstrap-4-table : Delete
    // 4. https://stackoverflow.com/a/37286676 : background color
    grid = $('#grid').grid({
        uiLibrary: 'bootstrap4',
        dataSource: '/Measurements',
        primaryKey: 'id',
        iconsLibrary: 'fontawesome',
        detailTemplate: '<div></div>',
        showHiddenColumnsAsDetails: true,
        responsive: true,
        notFoundText: 'Нет измерений',
        columns: [
            { field: 'id', title: '№', width: 45, type: 'number', priority: 1 },
            { field: 'F', title: 'F, Гц', width: 90, decimalDigits: 2, priority: 2 },
            { field: 'Rk', title: 'Rk, кОм', width: 90, decimalDigits: 1, priority: 2 },
            { field: 'Comment', title: 'Комментарий', editor: true, type: 'text', priority: 0 },
            { field: 'timestamp', title: 'Снято в', hidden: true, type: 'date', format: 'HH:MM:ss' },
            { field: 'F_deviation', title: 'ΔF, Гц', hidden: true, type: 'number', priority: 0, decimalDigits: 2 },
            { field: 'Rk_deviation', title: 'ΔRk, кОм', hidden: true, type: 'number', priority: 0, decimalDigits: 1 },
        ],
        pager: {
            leftControls: [
                $('<button type="button" class="btn btn-secondary" onclick="reset_session()"><i class="far fa-sticky-note"></i> Сброс</button>')
            ],
            rightControls: [
                $('<button type="button" class="btn btn-primary" onclick="add_res()"><i class="fas fa-plus"></i> Добавить</button>')
            ]
        },
        inlineEditing: {
            mode: 'dblclick'
        },
        dataBound: (_e, _data, _count) => {
            new BootstrapMenu("tr", {
                // Эта штука нужна чтобы получить "element_data" в методах ниже
                fetchElementData: function ($rowElem) {
                    var rowId = $rowElem.attr('data-position');
                    try {
                        return parseInt(rowId);
                    } catch {
                        return 0;
                    }
                },

                actionsGroups: [
                    ['ReMeasure'],
                    ['RemoveRow'],
                    ['InsertBefore']
                ],

                actions: {
                    ReMeasure: {
                        name: 'Снять заново',
                        iconClass: 'fa-solid fa-redo',
                        onClick: (row_id) => {
                            if (isNaN(row_id)) {
                                return;
                            }
                            console.log(`re-measure ${row_id}`);
                            add_res(row_id);
                        }
                    },
                    RemoveRow: {
                        name: 'Удалить',
                        iconClass: 'fas fa-trash',
                        onClick: (row_id) => {
                            if (isNaN(row_id)) {
                                return;
                            }
                            if (confirm(`Удалить измерение №${row_id}`)) {
                                grid.removeRow(row_id);
                                console.log(`remove ${row_id}`);
                            }
                        }
                    },
                    InsertBefore: {
                        name: 'Вставить перед',
                        iconClass: 'fas fa-plus',
                        onClick: (row_id) => {
                            if (isNaN(row_id)) {
                                return;
                            }
                            console.log(`insert before ${row_id}`);
                            add_res(row_id - 1, true);
                        }
                    }
                }
            });
        },
    }).on('rowRemoving', (e, $row, id, record: IResonatorData) => {
        console.log('row removing', e, $row, id, record);
        $.ajax({
            url: `/Measurements/${id}`,
            method: 'DELETE',
            success: () => {
                noty_success(`Измерение №${id} удалено.`);
            },
            error: (err) => {
                noty_error(err.responseText || err.statusText);
            },
        });
    }).on('cellDataChanged', (_e, _$cell, column, record: IResonatorData, newValue: string) => {
        console.log(`cell row=${record.id} comment to "${newValue}"`);
        $.ajax({
            url: `/Measurements/${record.id}`,
            method: 'PUT',
            data: newValue,
        });
    });

    // hotkeys
    /*
    hotkeys('right', (event, _handler) => {
        event.preventDefault();
        move_rel(-1);
    });
    */

    // report
    $('#date').datepicker({
        uiLibrary: 'bootstrap4',
        iconsLibrary: 'fontawesome',
        format: 'yyyy-mm-dd',
        value: new Date(Date.now()).toLocaleDateString(),
    });

    var dialog = $('#dialog').dialog({
        uiLibrary: 'bootstrap4',
        autoOpen: false,
        resizable: false,
        modal: true,
        width: 360,
    });

    MPdailog = $('#MPdailog').dialog({
        uiLibrary: 'bootstrap4',
        autoOpen: false,
        resizable: false,
        modal: true,
        width: 360,
    });

    $('#gen-report').on('click', (ev) => {
        $('#id').val('');
        $('#Name').val('');
        $('#PlaceOfBirth').val('');

        dialog.open('Создание отчета');

        ev.preventDefault();
    });

    $('#cancel_measure').on('click', cancel_measure);

    $('#btnSubmit').on('click', (e) => {
        // prevent form submission
        e.preventDefault();

        const form: JQuery<HTMLDivElement> = $('#report_form');
        const data_type = $(form).find("#data_type");
        const route_id = $(form).find("#route_id");
        const ambient_temperature_range = $(form).find("#ambient_temperature_range");
        const comment = $(form).find("#comment");
        const date = $(form).find("#date");

        const formErrors = validate({
            data_type: data_type.val(),
            route_id: route_id.val(),
            ambient_temperature_range: ambient_temperature_range.val(),
            date: date.val(),
        });

        const initialErros = {
            data_type: null,
            route_id: null,
            ambient_temperature_range: null,
            date: null,
        };

        if (formErrors?.error) {
            const { details } = formErrors.error;
            details.map((detail) => {
                initialErros[detail.context.key] = detail.message;
            });
        }

        // write the errors to the UI
        Object.keys(initialErros).map((errorName) => {
            if (initialErros[errorName] !== null) {
                // if the error exist
                // username input #username
                $(form).find(`#${errorName}`).removeClass("is-valid").addClass("is-invalid");
            } else {
                $(form).find(`#${errorName}`).removeClass("is-invalid");
            }
        });

        // to submit
        let isFormValid = Object.values(initialErros).every(
            (value) => value === null
        );

        if (isFormValid) {
            $.ajax({
                url: '/global',
                method: 'PUT',
                contentType: 'application/json; charset=utf-8',
                data: JSON.stringify({
                    data_type: data_type.val(),
                    route_id: route_id.val(),
                    ambient_temperature_range: ambient_temperature_range.val(),
                    date: date.val(),
                    comment: comment.val(),
                }),
                success: (_data) => {
                    noty_success('Отчет успешно создан.');
                    dialog.close();
                },
                error: (err) => {
                    noty_error(err.responseText || err.statusText);
                },
            }).then(() => {
                var link = document.createElement("a");
                // If you don't know the name or want to use
                // the webserver default set name = ''
                //link.setAttribute('download', "report.xlsx");
                link.href = '/report';
                document.body.appendChild(link);
                link.click();
                link.remove();
            });
        }
    });

    reload_global();

    $('#btnCancel').on('click', function () {
        dialog.close();
    });
});

function reload_global() {
    $.ajax({
        url: '/global',
        method: 'GET',
        dataType: 'json',
        success: (data) => {
            $('#data_type').val(data.data_type);
            $('#route_id').val(data.route_id);
            $('#ambient_temperature_range').val(data.ambient_temperature_range);
            $('#comment').val(data.comment);
            $('#date').val(data.date);
        }
    });
}

function validate(dataObject) {
    const result = schema.validate(
        {
            ...dataObject,
        },
        { abortEarly: false }
    );
    return result;
}

function reset_session() {
    $.ajax({
        url: '/global',
        method: 'DELETE',
        success: () => {
            grid && grid.reload();
            reload_global();
            noty_success('Сессия сброшена.');
        },
        error: (err) => {
            noty_error(err.responseText || err.statusText);
        },
    });
}

function add_res(id?: number, insertBefore: boolean = false) {
    var config: oboe.Options = {
        url: '',
        method: 'POST',
    };

    if (id === undefined) {
        config.url = '/Measurements';
    } else {
        config.url = `/Measurements/${id}`;
        config.body = insertBefore.toString();
    }

    MPdailog.open('Измерение');
    oboe(config)
        .done((data: IMeasureProcessStat) => {
            if (data.state == "Finished") {
                noty_success("Измерение завершено");
                MPdailog.close();
                grid.reload();
            }
            if (data.state == "Interrupted") {
                noty({
                    type: 'warning',
                    text: '<i class="fas fa-heart-broken"></i> Измерение отменено',
                    timeout: 3000
                });
                MPdailog.close();
            }

            if (data.state == "Running") {
                const freq_disp = $('#current-freq-display');
                if (data.freqs.length > 0) {
                    freq_disp.text(round_to_2_digits(data.freqs.pop()))
                    $('#current-freq-iqr-display')
                        .text(`${round_to_2_digits(data.freqs_avg.median)} ±${round_to_2_digits(data.freqs_avg.iqr)}`);
                } else {
                    freq_disp.text('---');
                }

                const rk_disp = $('#current-rk-display');
                if (data.rks.length > 0) {
                    rk_disp.text(round_to_2_digits(data.rks.pop()));
                    $('#current-rk-iqr-display')
                        .text(`${round_to_2_digits(data.rks_avg.median)} ±${round_to_2_digits(data.rks_avg.iqr)}`);
                } else {
                    rk_disp.text('---');
                }
            }
        }).fail((err: oboe.FailReason) => {
            noty_error(err.body || err.statusCode.toString());
            MPdailog.close();
        });
}

function cancel_measure() {
    $.ajax({
        url: '/Measurements',
        method: 'DELETE'
    });
}