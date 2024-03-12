
//-----------------------------------------------------------------------------

var notty: Noty | null = null;

// on page loaded jquery
$(() => {
    $('.adj-input').on('input', (ev) => {
        const $target = $(ev.target);
        const parameter = $target.prop('name');
        const value = parseFloat($target.val().toString());

        var data = {};
        data[parameter] = value;
        $.ajax({
            url: '/config',
            method: 'PATCH',
            data: JSON.stringify(data),
            contentType: 'application/json',
        }).then(() => {
            if (notty === null || notty.closed) {
                notty = noty_success('Чтобы установки вступили в силу, небхожимо перезапустить приложение.');
            }
        });
    });
});
