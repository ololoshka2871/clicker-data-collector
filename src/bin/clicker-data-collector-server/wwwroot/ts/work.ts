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
    frequency: string,
    rk: string,
    comment: String,
}

interface IState {
    // TODO
}

// ---------------------------------------------------------------------------------------------

let present_noty: Noty = null;

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
    $('#grid').grid({
        uiLibrary: 'bootstrap4',
        dataSource: '/Measurements',
        primaryKey: 'id',
        iconsLibrary: 'fontawesome',
        detailTemplate: '<div></div>',
        showHiddenColumnsAsDetails: true,
        responsive: true,
        columns: [
            { field: 'id', title: '№', width: 40, editor: true, type: 'number', priority: 1 },
            { field: 'F', title: 'F, Гц', width: 90, decimalDigits: 2, priority: 2 },
            { field: 'Rk', title: 'Rk, кОм', width: 90, decimalDigits: 1, priority: 2 },
            { field: 'Comment', title: 'Комментарий', editor: true, type: 'text', priority: 0 },
            { field: 'timestamp', title: 'Снято в:', hidden: true, type: 'date', format: 'HH:mm:ss', priority: 0 },
        ],
        pager: {
            leftControls: [],
            rightControls: [
                $('<button type="button" class="btn btn-primary" onclick="grid.reload()" class="btn btn-default">Добавить</button>')
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
                    return parseInt(rowId);
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
                        onClick: (element_data) => {
                            console.log('re-measure' + element_data);
                        }
                    },
                    RemoveRow: {
                        name: 'Удалить',
                        iconClass: 'fas fa-trash',
                        onClick: (element_data) => {
                            console.log('remove' + element_data);
                        }
                    },
                    InsertBefore: {
                        name: 'Вставить перед',
                        iconClass: 'fas fa-plus',
                        onClick: (element_data) => {
                            console.log('insert before' + element_data);
                        }
                    }
                }
            });
        }
    });

    // hotkeys
    /*
    hotkeys('right', (event, _handler) => {
        event.preventDefault();
        move_rel(-1);
    });
    */

    // report
    $('#gen-report').on('click', (ev) => {
        gen_report();
    });
});

function start_updater(chart: Chart) {
    oboe('/state')
        .done((state: IState) => {

        })
}