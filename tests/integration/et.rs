use assert_approx_eq::*;
use ctrl_lib::app_time::ctrl_time::CtrlTime;
use ctrl_lib::data_structs::sensor::metrics::evapo_transpiracao::*;

#[test]
fn test_et_1d() {
    let et_data = EtData {
        time: CtrlTime::from_utc_parts(2022, 7, 6, 15, 0, 0),
        lat: 50.8,
        elev: 100.,
        max_t: 21.5,
        min_t: 12.3,
        avg_hr: 52.,
        max_hr: 84.,
        min_hr: 63.,
        avg_ws: 10.,
        avg_press: 1001.,
    };

    let et = daily_evapo_transpiration(et_data);
    // divido por 0.6 porque a minha formula entre em linha de conta com o fator 0.6 em relação á formula original.
    assert_approx_eq!(et / 0.6, 5.5531, 0.0001);
}
