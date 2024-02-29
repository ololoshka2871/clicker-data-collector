// define interface for jquery JQuery<HTMLElement> where method tooltip is defined
interface JQuery<TElement extends Element = HTMLElement> extends Iterable<TElement> {
    tooltip(options?: any): JQuery<TElement>;
}

// declare hotkeys as defined global function
declare function hotkeys(key: string, callback: (event: KeyboardEvent, handler: any) => void): void;

// ---------------------------------------------------------------------------------------------

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

    const _tree = $('#tree');

    // ref: 
    // 1. https://gijgo.com/grid/demos/jquery-grid-material-design : New record
    // 2. https://gijgo.com/grid/demos/bootstrap-4-table : Delete
    _tree.grid({
        uiLibrary: 'bootstrap4',
        dataSource: '/Measurements/Get',
        primaryKey: 'index',
        iconsLibrary: 'fontawesome',
        columns: [
            { field: 'id', title: '№', sortable: true },
            { field: 'F', title: 'F, Гц', sortable: true },
            { field: 'Rk', title: 'Rk, кОм', sortable: true },
            { field: 'Comment', width: 300, title: 'Комментарий' },
        ],
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