"use strict";

var setup = function (model) {

    // OBJ SETUP
    this.btn_cfg_cancel_changes = document.getElementById("btn_cfg_cancel_changes");
    this.btn_cfg_save_changes = document.getElementById("btn_cfg_save_changes");

    this.config_no_connection_info = document.getElementById("config_no_connection_info");

    //Group S1
    this.cfg_live_since_text = document.getElementById("cfg_live_since");
    this.current_status_text = document.getElementById("cfg_status");

    this.btn_cfg_sync_databases = document.getElementById("btn_cfg_sync_databases");
    this.btn_cfg_reset_database = document.getElementById("btn_cfg_reset_database");

    this.btn_cfg_clean_accumulators = document.getElementById("btn_cfg_clean_accumulators");
    this.btn_cfg_retry_broker_connection = document.getElementById("btn_cfg_retry_broker_connection");

    //Group S2
    this.mqtt_web_broker_address_input = document.getElementById("mqtt_web_broker_address");
    this.mqtt_web_client_id_input = document.getElementById("mqtt_web_client_id");

    //Group S3
    this.max_sector_time_input = document.getElementById("max_sector_time");
    this.pump_recycle_time_input = document.getElementById("pump_recycle_time");
    this.schedule_def_text = document.getElementById("schedule_def");
    this.cfg_rain_amount_trigger_input = document.getElementById("cfg_rain_amount_trigger");
    this.cfg_wind_speed_trigger_input = document.getElementById("cfg_wind_speed_trigger");
    this.cfg_watering_suspend_timeout_input = document.getElementById("cfg_watering_suspend_timeout");
    this.cfg_decrease_alert_level_after_input = document.getElementById("cfg_decrease_alert_level_after");
    this.cfg_stress_control_interval = document.getElementById("cfg_stress_control_interval");

    //Group S4
    this.db_maint_days_input = document.getElementById("db_maint_days");
    this.db_maint_counter_input = document.getElementById("db_maint_counter");

    //Group S5
    this.precipitacao_acumulada_input = document.getElementById("weekly_acc_precipitation");
    this.rain_week_acc_days_input = document.getElementById("rain_week_acc_days");
    this.rain_week_acc_counter_input = document.getElementById("rain_week_acc_counter");
    
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
        let self = this;
        if (self._openedTab) self._closeTab(self.panels[self._openedTab]);
        slideDown(s.panel, "setup-coisas_open");
        show(s.close_btn);
        hide(s.open_btn);
        self._openedTab = s.panel.id;
        window.localStorage.setItem('ACTIVE_ACCORDION', self._openedTab);
        self = null;
    }.bind(this);

    //TESTED
    this._close_accordion = function (evt) {
        let self = this;
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_close"));
        let s = self.panels[tgt];
        self._closeTab(s);
        s = null;
        self = null;
    }.bind(this);

    //TESTED
    this._closeTab = function (s) {
        let self = this;
        slideUp(s.panel, "setup-coisas_open");
        hide(s.close_btn);
        show(s.open_btn);
        self._openedTab = undefined;
        window.localStorage.setItem('ACTIVE_ACCORDION', self._openedTab);
        self = null;
    }.bind(this);

    //TESTED
    this._toggle_accordion = function (evt) {
        let self = this;
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_header"));
        let s = self.panels[tgt];
        if (s.panel.clientHeight === 0) {
            self._open_tab(s);
        } else {
            self._closeTab(s);
        }
        s = null;
        self = null;
    }.bind(this);

    //TESTED
    this._hide_save_cancel_buttons = function () {
        let self = this;
        hide(self.btn_cfg_cancel_changes);
        hide(self.btn_cfg_save_changes);
        self = null;
    }.bind(this);

    // ui prep
    //TESTED
    this.disable_screen = function () {
        let self = this;

        show(self.config_no_connection_info);
        //Group S2
        disable(self.pump_recycle_time_input);
        disable(self.max_sector_time_input);
        disable(self.cfg_rain_amount_trigger_input);
        disable(self.cfg_wind_speed_trigger_input);
        disable(self.cfg_watering_suspend_timeout_input);
        disable(self.cfg_decrease_alert_level_after_input);
        disable(self.cfg_stress_control_interval);

        //Group S3
        disable(self.db_maint_days_input);
        disable(self.db_maint_counter_input);

        //Grupo S4
        disable(self.precipitacao_acumulada_input);
        disable(self.rain_week_acc_days_input);
        disable(self.rain_week_acc_counter_input);

        //Group S5
        disable(self.btn_cfg_sync_databases);
        disable(self.btn_cfg_reset_database);

        disable(self.btn_cfg_clean_accumulators);
        enable(self.btn_cfg_retry_broker_connection);
        self = null;
    }.bind(this);

    //TESTED
    this.enable_screen = function () {
        let self = this;

        hide(self.config_no_connection_info);
        //Group S2
        enable(self.pump_recycle_time_input);
        enable(self.max_sector_time_input);
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
        enable(self.btn_cfg_sync_databases);
        enable(self.btn_cfg_reset_database);

        enable(self.btn_cfg_clean_accumulators);
        disable(self.btn_cfg_retry_broker_connection);
        self = null;
    }.bind(this);

    // scather to screen
    //TESTED
    this._disable_dirty_listeners = function () {
        let self = this;
        if (!self._refreshing_screen_data) {
            self.refreshing_screen_data = true;
            clearListener("change", self.setup_dirty_mon);
        }
        self = null;
    }.bind(this);

    //TESTED
    this._enable_dirty_listeners = function () {
        let self = this;
        if (self._refreshing_screen_data) {
            addListener("change", self.setup_dirty_mon, self.setup_view_change);
            this._refreshing_screen_data = false;
        }
        self = null;
    }.bind(this);

    //TESTED
    this.render_local_data = function () {
        let self = this;

        self._disable_dirty_listeners();

        self.cfg["mqtt_web_broker_address"] = MQTT.endPoint.host;
        self.cfg["mqtt_web_client_id"] = MQTT.endPoint.client_id;

        setElementValue(self.mqtt_web_broker_address_input, MQTT.endPoint.host);
        setElementValue(self.mqtt_web_client_id_input, MQTT.endPoint.client_id);

        self._enable_dirty_listeners();
        self = null;
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

        //Group S2
        setElementValue(self.max_sector_time_input, cfg.max_sector_time);
        setElementValue(self.pump_recycle_time_input, cfg.pump_recycle_time);
        setElementValue(self.schedule_def_text, (cfg.schedule_def === 1) ? "Sim" : "Não");
        setElementValue(self.cfg_rain_amount_trigger_input, cfg.rain_alert_threshold);
        setElementValue(self.cfg_wind_speed_trigger_input, cfg.wind_alert_threshold);
        setElementValue(self.cfg_watering_suspend_timeout_input, cfg.watering_suspend_timeout);
        setElementValue(self.cfg_decrease_alert_level_after_input, cfg.decrease_alert_level_after);
        setElementValue(self.cfg_stress_control_interval, cfg.stress_control_interval);

        //Group S3
        setElementValue(self.db_maint_days_input, cfg.db_maint_days);
        setElementValue(self.db_maint_counter_input, cfg.db_maint_counter);

        //Group S4

        //Group S5
        setElementValue(self.cfg_live_since_text, cfg.live_since);
        setElementValue(self.current_status_text, cfg.current_state);

        self._enable_dirty_listeners();
        self = null;
    }.bind(this);

    //TESTED
    this.get_screen_data = function () {
        let self = this;
        let cfg = self.cfg;

        cfg.mqtt_web_broker_address = self.mqtt_web_broker_address_input.value;

        cfg.max_sector_time = parseFloat(self.max_sector_time_input.value);
        cfg.pump_recycle_time = parseFloat(self.pump_recycle_time_input.value);
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
        self = null;
    }.bind(this);

    // events & listeners
    //TESTED
    this.setup_view_change = function setup_view_change(target) {
        let self = this;
        show(self.btn_cfg_cancel_changes);
        show(self.btn_cfg_save_changes);
        self = null;
    }.bind(this);

    this.btn_cfg_cancel_changes.onclick = function () {
        let self = this;
        self._hide_save_cancel_buttons();
        //repor valores no ecran
        self._disable_dirty_listeners();
        self.render();
        self._enable_dirty_listeners();
        self = null;
    }.bind(this);

    this._assign = function (fromModel, toModel) {
        toModel.max_sector_time = fromModel.max_sector_time;
        toModel.pump_recycle_time = fromModel.pump_recycle_time;
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
        toModel.last_change = fromModel.last_change;
    }.bind(this);

    //TESTED
    this.save = function () {
        let self = this;
        let cfg = self.cfg;
        let endPoint = MQTT.endPoint;
        let config = DB.config;

        let reconnect = (endPoint.host !== cfg.mqtt_web_broker_address) ||
            (endPoint.client_id !== cfg.mqtt_web_client_id);

        try {
            if (reconnect) {
                endPoint.host = cfg.mqtt_web_broker_address;
                endPoint.client_id = cfg.mqtt_web_client_id;
                endPoint.save();
                MQTT._configureClientOptions();
                MQTT._connect();

                //REVIEW TEST THIS
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

    // gather info from screen
    //TESTED
    this.btn_cfg_save_changes.onclick = function () {
        let self = this;
        self._hide_save_cancel_buttons();
        self.get_screen_data();
        self.save();

        try {
            // send info
            DB.syncToServer();
        } catch (err) {
            log.error(err);
        } finally {
            self._enable_dirty_listeners();
        }
        self = null;
    }.bind(this);

    this.scather = function(){
        let self = this;
        self._hide_save_cancel_buttons();
        self.get_screen_data();
        self = null;
    }.bind(this);

    //TESTED
    this.btn_cfg_sync_databases.onclick = function () {
        let self = this;
        try {
            self.scather();
            self.save();
            DB.syncToServer();
            DB.requestFullsyncFromServer();
        } catch (err) {
            log.error(err);
        }
        self = null;
    }.bind(this);

    //TESTED
    this.btn_cfg_reset_database.onclick = function () {
        let self = this;
        try {
            self.scather();
            self.save();
            DB.requestFullsyncFromServer();
        } catch (err) {
            log.error(err);
        }
        self = null;
    }.bind(this);

    //TESTED
    this.btn_cfg_clean_accumulators.onclick = function () {
        let self = this;
        try {
            self.scather();
            self.cfg.db_maint_counter = 0.0;
            self.cfg.rain_week_acc_counter = 0.0;
            self.cfg.weekly_acc_precipitation = 0.0;
            self.save();
            self.render();
            DB.syncToServer();
        } catch (err) {
            log.error(err);
        }
        self = null;
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

    this.connected_event = function (param) {
        try {
            this.enable_screen();
        } catch (err) {
            log.error(err);
        }
    }.bind(this);

    this.disconnected_event = function (param) {
        try {
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
                // new info arrived
                // validate what changed in relation with any changes made locally in the screen
                // Update the things that were not changed in the screen
                // Identify the things updated in the backend, that also were changes in the screen - in the footer CARD 
                // "Informação que foi alterada foi atualizada pelo servidor - quer atualizar já a informação e perder as alterações?""
                // give a list of the changed info - to help in the decision
                // "no" answer its of no use, because new info will arrive and update the new info without warning
                // only happens when changing counters
                if (this._getDiff(cfg, newData, CONFIG_ATTRIBUTES_UPDATABLE, ["last_change"])) {
                    // counters changes from the backend - give feedback to the user
                    // if no answer within the timeout time, update the info
                    var data = {
                        message: 'Existiram alterações no servidor durante a edição.  É para esmagar as alterações?',
                        timeout: 500,
                        actionHandler: self.override_handler,
                        actionText: 'Sim!'
                    };
                    if (MAIN_VIEW._menu_name === "coisas-geral" && MAIN_VIEW._menu_name === "coisas-panel") {
                        // follow user instruction
                        MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.showSnackbar(data);
                    } else {
                        // update info
                        this.override_handler();
                        rendered = true;
                    }
                } else {
                    // no changes and no render - nothing to do
                }
                cfg.live_since = newData.live_since;
                cfg.current_state = newData.current_state;
                if (!rendered) self.render();
            }
        } catch (err) {
            log.error(err);
        }
        self = null;
    }.bind(this);

    //TESTED
    this.override_handler = function (event) {
        let self = this;
        let cfg = self.cfg;
        let newData = self.newData;

        self._assign(newData, cfg);
        self.save();
        if (MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.active)
            MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.cleanup_();

        self.render();
        self = null;
    }.bind(this);

    this.cfg = model.clone();

    this._refreshing_screen_data = false;
    this.newData = null;

    this._openedTab = undefined;

    this.setup_dirty_mon = [];
    //local
    this.setup_dirty_mon.push(this.mqtt_web_broker_address_input);

    this.setup_dirty_mon.push(this.max_sector_time_input);
    this.setup_dirty_mon.push(this.pump_recycle_time_input);
    this.setup_dirty_mon.push(this.cfg_rain_amount_trigger_input);
    this.setup_dirty_mon.push(this.cfg_wind_speed_trigger_input);
    this.setup_dirty_mon.push(this.cfg_watering_suspend_timeout_input);
    this.setup_dirty_mon.push(this.cfg_decrease_alert_level_after_input);
    this.setup_dirty_mon.push(this.db_maint_days_input);
    this.setup_dirty_mon.push(this.db_maint_counter_input);
    this.setup_dirty_mon.push(this.precipitacao_acumulada_input);
    this.setup_dirty_mon.push(this.rain_week_acc_days_input);
    this.setup_dirty_mon.push(this.rain_week_acc_counter_input);
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

    let obj = null;
    for (let key in this.panels) {
        obj = this.panels[key];
        obj.open_btn.onclick = this._open_accordion;
        obj.close_btn.onclick = this._close_accordion;
        obj.accordion.onclick = this._toggle_accordion;
    }
    obj = null;

    //screen init configs
    this._hide_save_cancel_buttons();
    enable(this.btn_cfg_retry_broker_connection);

    addListener("change", this.setup_dirty_mon, this.setup_view_change);

    DB.newDataEvent.registerObserver(this, this.notifyNewData);
    CTRL_CONN.connected_event.registerObserver(this, this.connected_event);
    CTRL_CONN.disconnected_event.registerObserver(this, this.disconnected_event);
    if (MQTT.haveController) {
        this.notifyConnection(true);
    }

    this._openedTab = window.localStorage.getItem('ACTIVE_ACCORDION');
    if (this._openedTab) {
        let s = this.panels[this._openedTab];
        this._openedTab = undefined;
        this._open_tab(s);
        s = null;
    }
};
