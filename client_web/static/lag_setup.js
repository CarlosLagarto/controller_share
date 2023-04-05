"use strict";

var setup = function (model) {

    // SETUP DO OBJETO
    this.btn_cfg_cancel_changes = document.getElementById("btn_cfg_cancel_changes");
    this.btn_cfg_save_changes = document.getElementById("btn_cfg_save_changes");

    this.config_no_connection_info = document.getElementById("config_no_connection_info");

    //Grupo S1
    this.cfg_live_since_text = document.getElementById("cfg_live_since");
    this.shutdown_switch = document.getElementById("cfg_shutdown");
    this.current_status_text = document.getElementById("cfg_status");
    this.simulation_check = document.getElementById("cfg_simulation");

    this.btn_cfg_sync_databases = document.getElementById("btn_cfg_sync_databases");
    this.btn_cfg_reset_database = document.getElementById("btn_cfg_reset_database");

    this.btn_cfg_clean_accumulators = document.getElementById("btn_cfg_clean_accumulators");
    this.btn_cfg_retry_broker_connection = document.getElementById("btn_cfg_retry_broker_connection");

    //Grupo S2
    this.mqtt_web_broker_address_input = document.getElementById("mqtt_web_broker_address");
    this.mqtt_web_broker_port_input = document.getElementById("mqtt_web_broker_port");
    this.mqtt_web_client_id_input = document.getElementById("mqtt_web_client_id");

    //Grupo S3
    this.max_sector_time_input = document.getElementById("max_sector_time");
    this.pump_recycle_time_input = document.getElementById("pump_recycle_time");
    this.schedule_def_text = document.getElementById("schedule_def");
    this.cfg_week_water_amount_input = document.getElementById("cfg_week_water_amount");
    this.cfg_week_watering_times_input = document.getElementById("cfg_week_watering_times");
    this.cfg_grass_root_length_input = document.getElementById("cfg_grass_root_length");
    this.cfg_rain_amount_trigger_input = document.getElementById("cfg_rain_amount_trigger");
    this.cfg_wind_speed_trigger_input = document.getElementById("cfg_wind_speed_trigger");
    this.cfg_watering_suspend_timeout_input = document.getElementById("cfg_watering_suspend_timeout");
    this.cfg_decrease_alert_level_after_input = document.getElementById("cfg_decrease_alert_level_after");
    this.cfg_stress_control_interval = document.getElementById("cfg_stress_control_interval");

    //Grupo S4
    this.db_maint_days_input = document.getElementById("db_maint_days");
    this.db_maint_counter_input = document.getElementById("db_maint_counter");

    //Grupo S5
    this.precipitacao_acumulada_input = document.getElementById("weekly_acc_precipitation");
    this.rain_week_acc_days_input = document.getElementById("rain_week_acc_days");
    this.rain_week_acc_counter_input = document.getElementById("rain_week_acc_counter");

    this.cfg = model.clone();

    this._refreshing_screen_data = false;
    this.newData = null;

    this._openedTab = undefined;

    this.setup_dirty_mon = [];
    //local
    this.setup_dirty_mon.push(this.mqtt_web_broker_address_input);
    this.setup_dirty_mon.push(this.mqtt_web_broker_port_input);

    this.setup_dirty_mon.push(this.max_sector_time_input);
    this.setup_dirty_mon.push(this.pump_recycle_time_input);
    this.setup_dirty_mon.push(this.cfg_week_water_amount_input);
    this.setup_dirty_mon.push(this.cfg_week_watering_times_input);
    this.setup_dirty_mon.push(this.cfg_grass_root_length_input);
    this.setup_dirty_mon.push(this.cfg_rain_amount_trigger_input);
    this.setup_dirty_mon.push(this.cfg_wind_speed_trigger_input);
    this.setup_dirty_mon.push(this.cfg_watering_suspend_timeout_input);
    this.setup_dirty_mon.push(this.cfg_decrease_alert_level_after_input);
    this.setup_dirty_mon.push(this.db_maint_days_input);
    this.setup_dirty_mon.push(this.db_maint_counter_input);
    this.setup_dirty_mon.push(this.precipitacao_acumulada_input);
    this.setup_dirty_mon.push(this.rain_week_acc_days_input);
    this.setup_dirty_mon.push(this.rain_week_acc_counter_input);
    this.setup_dirty_mon.push(this.shutdown_switch);
    this.setup_dirty_mon.push(this.cfg_stress_control_interval);

    this.panels = {
        "setup-group-2": {
            panel: document.getElementById("setup-group-2"),
            accordion: document.getElementById("setup-group-2_header"),
            open_btn: document.getElementById("setup-group-2_open"),
            close_btn: document.getElementById("setup-group-2_close")
        },
        "setup-group-3": {
            panel: document.getElementById("setup-group-3"),
            accordion: document.getElementById("setup-group-3_header"),
            open_btn: document.getElementById("setup-group-3_open"),
            close_btn: document.getElementById("setup-group-3_close")
        },
        "setup-group-4": {
            panel: document.getElementById("setup-group-4"),
            accordion: document.getElementById("setup-group-4_header"),
            open_btn: document.getElementById("setup-group-4_open"),
            close_btn: document.getElementById("setup-group-4_close")
        },
        "setup-group-5": {
            panel: document.getElementById("setup-group-5"),
            accordion: document.getElementById("setup-group-5_header"),
            open_btn: document.getElementById("setup-group-5_open"),
            close_btn: document.getElementById("setup-group-5_close")
        }
    };
    //TESTED
    this._open_accordion = function (evt) {
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_open"));
        let s = this.panels[tgt];
        this._open_tab(s);
        s = null;
    }.bind(this);

    //TESTED
    this._open_tab = function (s) {
        if (this._openedTab) this._closeTab(this.panels[this._openedTab]);
        slideDown(s.panel, "setup-coisas_open");
        show(s.close_btn);
        hide(s.open_btn);
        this._openedTab = s.panel.id;
        window.localStorage.setItem('ACTIVE_ACCORDION', this._openedTab);
    }.bind(this);

    //TESTED
    this._close_accordion = function (evt) {
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_close"));
        let s = this.panels[tgt];
        this._closeTab(s);
        s = null;
    }.bind(this);

    //TESTED
    this._closeTab = function (s) {
        slideUp(s.panel, "setup-coisas_open");
        hide(s.close_btn);
        show(s.open_btn);
        this._openedTab = undefined;
        window.localStorage.setItem('ACTIVE_ACCORDION', this._openedTab);
    }.bind(this);

    //TESTED
    this._toggle_accordion = function (evt) {
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_header"));
        let s = this.panels[tgt];
        if (s.panel.clientHeight === 0) {
            this._open_tab(s);
        } else {
            this._closeTab(s);
        }
        s = null;
    }.bind(this);

    //TESTED
    this._hide_save_cancel_buttons = function () {
        hide(this.btn_cfg_cancel_changes);
        hide(this.btn_cfg_save_changes);
    }.bind(this);

    // PREPARAÇÃO DO ECRAN
    //TESTED
    this.disable_screen = function () {
        let self = this;

        show(self.config_no_connection_info);
        //Grupo S2
        disable(self.pump_recycle_time_input);
        disable(self.max_sector_time_input);
        disable(self.cfg_week_water_amount_input);
        disable(self.cfg_week_watering_times_input);
        disable(self.cfg_grass_root_length_input);
        disable(self.cfg_rain_amount_trigger_input);
        disable(self.cfg_wind_speed_trigger_input);
        disable(self.cfg_watering_suspend_timeout_input);
        disable(self.cfg_decrease_alert_level_after_input);
        disable(self.cfg_stress_control_interval);

        //grupo S3
        disable(self.db_maint_days_input);
        disable(self.db_maint_counter_input);

        //Grupo S4
        disable(self.precipitacao_acumulada_input);
        disable(self.rain_week_acc_days_input);
        disable(self.rain_week_acc_counter_input);

        //Grupo S5
        disableMaterialSwitch(self.shutdown_switch);

        disable(self.btn_cfg_sync_databases);
        disable(self.btn_cfg_reset_database);

        disable(self.btn_cfg_clean_accumulators);
        enable(self.btn_cfg_retry_broker_connection);
    }.bind(this);

    //TESTED
    this.enable_screen = function () {
        let self = this;

        hide(self.config_no_connection_info);
        //Grupo S2
        enable(self.pump_recycle_time_input);
        enable(self.max_sector_time_input);
        enable(self.cfg_week_water_amount_input);
        enable(self.cfg_week_watering_times_input);
        enable(self.cfg_grass_root_length_input);
        enable(self.cfg_rain_amount_trigger_input);
        enable(self.cfg_wind_speed_trigger_input);
        enable(self.cfg_watering_suspend_timeout_input);
        enable(self.cfg_decrease_alert_level_after_input);
        enable(self.cfg_stress_control_interval);

        // S3
        enable(self.db_maint_days_input);
        enable(self.db_maint_counter_input);

        //S4
        enable(self.precipitacao_acumulada_input);
        enable(self.rain_week_acc_days_input);
        enable(self.rain_week_acc_counter_input);

        //S5
        enableMaterialSwitch(self.shutdown_switch);

        enable(self.btn_cfg_sync_databases);
        enable(self.btn_cfg_reset_database);

        enable(self.btn_cfg_clean_accumulators);
        disable(self.btn_cfg_retry_broker_connection);
    }.bind(this);

    // RENDER DO OBJETO PARA O ECRAN
    //TESTED
    this._disable_dirty_listeners = function () {
        if (!this._refreshing_screen_data) {
            this.refreshing_screen_data = true;
            clearListener("change", this.setup_dirty_mon);
        }
    }.bind(this);

    //TESTED
    this._enable_dirty_listeners = function () {
        if (this._refreshing_screen_data) {
            addListener("change", this.setup_dirty_mon, this.setup_view_change);
            this._refreshing_screen_data = false;
        }
    }.bind(this);

    //TESTED
    this.render_local_data = function () {
        let self = this;

        self._disable_dirty_listeners();

        self.cfg["mqtt_web_broker_address"] = MQTT.endPoint.host;
        self.cfg["mqtt_web_broker_port"] = MQTT.endPoint.port;
        self.cfg["mqtt_web_client_id"] = MQTT.endPoint.client_id;

        setElementValue(self.mqtt_web_broker_address_input, MQTT.endPoint.host);
        setElementValue(self.mqtt_web_broker_port_input, MQTT.endPoint.port);
        setElementValue(self.mqtt_web_client_id_input, MQTT.endPoint.client_id);

        self._enable_dirty_listeners();
    }.bind(this);

    //TESTED
    this.set_simulation_check = function (simulation_value) {
        if (simulation_value === 0) uncheckMaterialCheckbox(this.simulation_check);
        else checkMaterialCheckbox(this.simulation_check);
    }.bind(this);

    // TESTED
    this.render = function () {
        let self = this;
        let cfg = self.cfg;

        self._disable_dirty_listeners();

        self.render_local_data();

        if (MQTT.isDisconnected()) self.disable_screen();

        //Grupo S2
        setElementValue(self.max_sector_time_input, cfg.max_sector_time);
        setElementValue(self.pump_recycle_time_input, cfg.pump_recycle_time);
        setElementValue(self.schedule_def_text, (cfg.schedule_def === 1) ? "Sim" : "Não");
        setElementValue(self.cfg_week_water_amount_input, cfg.strategy_water_mm_per_week);
        setElementValue(self.cfg_week_watering_times_input, cfg.strategy_water_days_per_week);
        setElementValue(self.cfg_grass_root_length_input, cfg.strategy_root_length);
        setElementValue(self.cfg_rain_amount_trigger_input, cfg.rain_alert_threshold);
        setElementValue(self.cfg_wind_speed_trigger_input, cfg.wind_alert_threshold);
        setElementValue(self.cfg_watering_suspend_timeout_input, cfg.watering_suspend_timeout);
        setElementValue(self.cfg_decrease_alert_level_after_input, cfg.decrease_alert_level_after);
        setElementValue(self.cfg_stress_control_interval, cfg.stress_control_interval);

        //Grupo S3
        setElementValue(self.db_maint_days_input, cfg.db_maint_days);
        setElementValue(self.db_maint_counter_input, cfg.db_maint_counter);

        //Grupo S4
        setElementValue(self.precipitacao_acumulada_input, cfg.weekly_acc_precipitation);
        setElementValue(self.rain_week_acc_days_input, cfg.rain_week_acc_days);
        setElementValue(self.rain_week_acc_counter_input, cfg.rain_week_acc_counter);

        //Grupo S5
        setElementValue(self.cfg_live_since_text, cfg.live_since);
        setElementValue(self.current_status_text, cfg.current_state);
        self.set_simulation_check(cfg.simulation);

        self._enable_dirty_listeners();
    }.bind(this);

    //TESTED
    this.get_screen_data = function () {
        let self = this;
        let cfg = self.cfg;

        cfg.mqtt_web_broker_address = self.mqtt_web_broker_address_input.value;
        cfg.mqtt_web_broker_port = parseInt(self.mqtt_web_broker_port_input.value);

        cfg.max_sector_time = parseFloat(self.max_sector_time_input.value);
        cfg.pump_recycle_time = parseFloat(self.pump_recycle_time_input.value);
        cfg.strategy_water_mm_per_week = parseFloat(self.cfg_week_water_amount_input.value);
        cfg.strategy_water_days_per_week = parseInt(self.cfg_week_watering_times_input.value);
        cfg.strategy_root_length = parseInt(self.cfg_grass_root_length_input.value);
        cfg.rain_alert_threshold = parseFloat(self.cfg_rain_amount_trigger_input.value);
        cfg.wind_alert_threshold = parseFloat(self.cfg_wind_speed_trigger_input.value);
        cfg.watering_suspend_timeout = parseFloat(self.cfg_watering_suspend_timeout_input.value);
        cfg.decrease_alert_level_after = parseFloat(self.cfg_decrease_alert_level_after_input.value);
        cfg.stress_control_interval = parseFloat(self.cfg_stress_control_interval.value);

        cfg.db_maint_days = parseInt(self.db_maint_days_input.value);
        cfg.db_maint_counter = parseInt(self.db_maint_counter_input.value);

        cfg.weekly_acc_precipitation = parseInt(self.precipitacao_acumulada_input.value);
        cfg.rain_week_acc_days = parseInt(self.rain_week_acc_days_input.value);
        cfg.rain_week_acc_counter = parseInt(self.rain_week_acc_counter_input.value);
    }.bind(this);

    // eventos e listeners
    //TESTED
    this.setup_view_change = function setup_view_change(target) {
        show(this.btn_cfg_cancel_changes);
        show(this.btn_cfg_save_changes);
    }.bind(this);

    this.btn_cfg_cancel_changes.onclick = function () {
        this._hide_save_cancel_buttons();
        //repor valores no ecran
        this._disable_dirty_listeners();
        uncheckMaterialSwitch(this.shutdown_switch);
        this.render();
        this._enable_dirty_listeners();
    }.bind(this);

    this._assign = function (fromModel, toModel) {
        toModel.max_sector_time = fromModel.max_sector_time;
        toModel.pump_recycle_time = fromModel.pump_recycle_time;
        toModel.strategy_water_mm_per_week = fromModel.strategy_water_mm_per_week;
        toModel.strategy_water_days_per_week = fromModel.strategy_water_days_per_week;
        toModel.strategy_root_length = fromModel.strategy_root_length;
        toModel.rain_alert_threshold = fromModel.rain_alert_threshold;
        toModel.wind_alert_threshold = fromModel.wind_alert_threshold;
        toModel.watering_suspend_timeout = fromModel.watering_suspend_timeout;
        toModel.decrease_alert_level_after = fromModel.decrease_alert_level_after;
        toModel.stress_control_interval = fromModel.stress_control_interval;
        toModel.db_maint_days = fromModel.db_maint_days;
        toModel.db_maint_counter = fromModel.db_maint_counter;
        toModel.weekly_acc_precipitation = fromModel.weekly_acc_precipitation;
        toModel.rain_week_acc_days = fromModel.rain_week_acc_days;
        toModel.rain_week_acc_counter = fromModel.rain_week_acc_counter;
        toModel.simulation = fromModel.simulation_check;
        toModel.last_change = fromModel.last_change;
    }.bind(this);

    //TESTED
    this.save = function () {
        let self = this;
        let cfg = self.cfg;
        let endPoint = MQTT.endPoint;
        let config = DB.config;

        let reconnect = (endPoint.host !== cfg.mqtt_web_broker_address) ||
            (endPoint.port !== cfg.mqtt_web_broker_port) ||
            (endPoint.client_id !== cfg.mqtt_web_client_id);

        try {
            if (reconnect) {
                endPoint.host = cfg.mqtt_web_broker_address;
                endPoint.port = cfg.mqtt_web_broker_port;
                endPoint.client_id = cfg.mqtt_web_client_id;
                endPoint.save();
                MQTT._connect();

                //REVER testar isto como deve ser
                var waitForReconnection = timeoutms => new Promise((resolve, reject) => {
                    var check = () => {
                        if (!MQTT.isDisconnected())
                            resolve();
                        else if ((timeoutms -= 100) < 0) {
                            //reject('timed out!');
                            resolve()
                        } else
                            setTimeout(check, 100);
                    };
                    setTimeout(check, 100);
                });

                waitForReconnection(2000)
            }
            this._assign(cfg, config);
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    //grava a informação do ecran setup
    //TESTED
    this.btn_cfg_save_changes.onclick = function () {
        let self = this;
        self._hide_save_cancel_buttons();
        self.get_screen_data();
        self.save();

        try {
            //envia a info
            DB.syncToServer();

            if (self.shutdown_switch.checked) { //se estiver ativo
                //vai enviar a mensagem de shutdown
                self._disable_dirty_listeners();
                buildAndSendMessage(MSG.T.CTS_STATUS_SHUTDOWN, "");
            }
        } catch (err) {
            log.error(err);
        } finally {
            uncheckMaterialSwitch(self.shutdown_switch);
            self._enable_dirty_listeners();
        }
    }.bind(this);

    //TESTED
    this.btn_cfg_sync_databases.onclick = function () {
        try {
            this._hide_save_cancel_buttons();
            this.get_screen_data();
            this.save();
            DB.syncToServer();
            DB.requestFullsyncFromServer();
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    //TESTED
    this.btn_cfg_reset_database.onclick = function () {
        try {
            this._hide_save_cancel_buttons();
            this.get_screen_data();
            this.save();
            DB.requestFullsyncFromServer();
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    //TESTED
    this.btn_cfg_clean_accumulators.onclick = function () {
        let self = this;
        try {
            self._hide_save_cancel_buttons();
            self.get_screen_data();
            self.cfg.db_maint_counter = 0.0;
            self.cfg.rain_week_acc_counter = 0.0;
            self.cfg.weekly_acc_precipitation = 0.0;
            self.save();
            self.render();
            DB.syncToServer();
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    //TESTED
    this.btn_cfg_retry_broker_connection.onclick = function () {
        try {
            if (MQTT.isDisconnected()) {
                MQTT._connect();
            }
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    this.notifyConnection = function (status) {
        try {
            if (status)
                this.enable_screen();
            else
                this.disable_screen();
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    this._getDiff = function (o1, o2, attr_list, exclude_list) {
        let diff = false;
        for (let attr in attr_list) {
            if (exclude_list.indexOf(attr)) {
                if (o1[attr] !== o2[attr]) {
                    diff = true;
                    break;
                }
            }
        }
        return diff;
    }.bind(this);

    //TESTED
    this.notifyNewData = function (newDataObj) {
        let self = this;
        let cfg = self.cfg;
        let newData = null;
        let rendered = false;
        try {
            if (newDataObj.type === "config") {
                self.newData = newDataObj.data;
                newData = self.newData;
                // CHEGOU INFO NOVA
                // VALIDAR O QUE É QUE MUDOU EM RELAÇÃO ÁS ALTERAÇÕES EFETUADAS NO ECRAN.
                // ALTERAR AS COISAS QUE NÃO FORAM ALTERADAS.
                // IDENTIFICAR AS OUTRAS E INFORMAR O UTILIZADOR - NUM CARD COM ACÇÃO PERTO DO FOOTER
                // Informação que foi alterada foi atualizada pelo servidor - quer atualizar já a informação e perder as alterações?
                // dar lista da informação que ficou diferente - para dar pista sobre o que mudou e auxiliar a decisão
                // a resposta "não" tipicamente servirá de pouco, porque nova informação chegará quando não houver alterações e esmagar a info sem avisar.
                // a não ser nos casos de acertos dos contadores....e percebi agora que isto só se verifica ai, porque o resto é mesmo configuração...
                //...check :-)  no ecran de configuração temos info de configuração :-)
                if (this._getDiff(cfg, newData, CONFIG_ATTRIBUTES_UPDATABLE, ["last_change"])) {
                    //houve alterações - em tese só os contadores podem mudar - vamos ser simpáticos e dar feedback ao utilizador
                    // se não responder, porque afinal não estava a olhar para o ecran, esmagamos a info.
                    // neste caso esta alteração só entra em vigor após um restart.
                    // REVIEW falta testar a função de restart, que em tese já funciona do lado do servidor.
                    var data = {
                        message: 'Existiram alterações no servidor durante a edição.  É para esmagar as alterações?',
                        timeout: 500,
                        actionHandler: self.override_handler,
                        actionText: 'Sim!'
                    };
                    if (MAIN_VIEW._menu_name === "coisas-geral" && MAIN_VIEW._menu_name === "coisas-panel") {
                        //atualizamos ou não em função da resposta do user
                        // noinspection JSUnresolvedFunction
                        MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.showSnackbar(data);
                    } else {
                        //não está a acontecer nada no ecran de setup pelo que atualizamos a info
                        this.override_handler();
                        rendered = true;
                    }
                } else {
                    //se não houve alterações, nem render precisamos fazer
                }
                cfg.live_since = newData.live_since;
                cfg.current_state = newData.current_state;
                if (!rendered) self.render();
            }
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    //TESTED
    this.override_handler = function (event) {
        let self = this;
        let cfg = self.cfg;
        let newData = self.newData;

        this._assign(newData, cfg);

        self.save();
        if (MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.active)
            MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.cleanup_();

        self.render();
    }.bind(this);

    let obj = null;
    for (let key in this.panels) {
        obj = this.panels[key];
        obj.open_btn.onclick = this._open_accordion;
        obj.close_btn.onclick = this._close_accordion;
        obj.accordion.onclick = this._toggle_accordion;
    }
    obj = null;

    //configs iniciais do ecran
    this._hide_save_cancel_buttons();
    enable(this.btn_cfg_retry_broker_connection);

    addListener("change", this.setup_dirty_mon, this.setup_view_change);

    DB.newDataEvent.registerObserver(this);
    MQTT.connectionEvent.registerObserver(this);
    if (MQTT.haveController) {
        this.notifyConnection(true);
    }

    this._openedTab = window.localStorage.getItem('ACTIVE_ACCORDION');
    if (this._openedTab) {
        let s = this.panels[this._openedTab];
        this._openedTab = undefined;
        this._open_tab(s);
    }
};
