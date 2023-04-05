"use strict";

const DEVICE_TYPE = {
    RELAY: 0,  // on/off element types
    ROLLER: 1, // click events of the 3 icons element type
    TRIGGER_SWITCH: 2, // click event of icon element type
}

const CENA_TYPE = {
    ACTUATOR: 0,
    SCENE: 1,
}

var command = function(id, cmd){
    this.cmd = cmd;
    this.device_id = id;
}

const COMMAND = {
    NONE:  1,
    OFF:   2,
    ON:    4,
    UP:    8,
    STOP: 16,
    DOWN: 32,
}

const ELEMENT_TYPE = {
    RADIO:0,
    ICON:1,
}

var element = function(id, cena_type, obj_type, element_type ){
    this.id = id;
    this.cena_type = cena_type;
    this.dom_element = document.getElementById(id);
    this.obj_type = obj_type;
    this.element_type = element_type;
    // indexed by COMMAND [null,null,null,null,null]
    this.command_elements = [null,null,null,null,null];

    this.device_cmd = function(cmd){
        let self = this;
        switch (self.obj_type){
            case DEVICE_TYPE.RELAY:
                return new command(self.id, cmd);
                break;
            case DEVICE_TYPE.ROLLER:
                return new command(self.id, cmd);
                break;
            case DEVICE_TYPE.TRIGGER_SWITCH:
                return new command(self.id, cmd);
                break;                
        }
        
        return null;
    }.bind(this);


    // this.cmd_on = function(evt){
    //     // let id = evt.currentTarget.id;
    //     // let tgt = id.slice(0, id.indexOf("_open"));
    //     // this._open_tab(tgt);
    // }.bind(this);

    // this.cmd_off = function(evt){
    //     // let id = evt.currentTarget.id;
    //     // let tgt = id.slice(0, id.indexOf("_open"));
    //     // this._open_tab(tgt);
    // }.bind(this);

    this.cmd_up = function(evt){
        // let id = evt.currentTarget.id;
        // let tgt = id.slice(0, id.indexOf("_open"));
        // this._open_tab(tgt);
    }.bind(this);

    this.cmd_down = function(evt){
        // let id = evt.currentTarget.id;
        // let tgt = id.slice(0, id.indexOf("_open"));
        // this._open_tab(tgt);
    }.bind(this);

    this.cmd_stop = function(evt){
        // let id = evt.currentTarget.id;
        // let tgt = id.slice(0, id.indexOf("_open"));
        // this._open_tab(tgt);
    }.bind(this);

    // this.cmd_change = function(evt){

    // }.bind(this);

    this.add_event_handlers = function(off_callback, on_callback, up_callback, stop_callback, down_callback){
        let self = this;
        switch (self.obj_type){
            case DEVICE_TYPE.RELAY:
                if (self.element_type === ELEMENT_TYPE.RADIO){
                    self.command_elements[COMMAND.OFF] = document.getElementById(self.id + "_dis");
                    self.command_elements[COMMAND.ON] = document.getElementById(self.id + "_ena");
                    self.command_elements[COMMAND.OFF].onchange = off_callback;
                    self.command_elements[COMMAND.ON].onchange = on_callback;
                    self.command_elements[COMMAND.UP] = null;
                    self.command_elements[COMMAND.STOP] = null;
                    self.command_elements[COMMAND.DOWN] = null;
                };
                if (self.element_type === ELEMENT_TYPE.ICON){
                    // still no use case for this
                }
                break;
            case DEVICE_TYPE.ROLLER:
                if (self.element_type == ELEMENT_TYPE.ICON){
                    self.command_elements[COMMAND.OFF] = null;
                    self.command_elements[COMMAND.ON] = null;
                    self.command_elements[COMMAND.UP] = document.getElementById(self.id + "_open");
                    self.command_elements[COMMAND.UP].onclick = self.cmd_up;
                    self.command_elements[COMMAND.STOP] = document.getElementById(self.id + "_stop");
                    self.command_elements[COMMAND.STOP].onclick = self.cmd_stop;
                    self.command_elements[COMMAND.DOWN] = document.getElementById(self.id + "_close");
                    self.command_elements[COMMAND.DOWN].onclick = self.cmd_down;
                }
                break;
            case DEVICE_TYPE.TRIGGER_SWITCH:
                if (self.element_type == ELEMENT_TYPE.ICON){
                    self.command_elements[COMMAND.OFF] = null;
                    self.command_elements[COMMAND.ON] = document.getElementById(self.id + "_trigger");
                    self.command_elements[COMMAND.ON].onclick = on_callback;
                    self.command_elements[COMMAND.UP] = null;
                    self.command_elements[COMMAND.STOP] = null;
                    self.command_elements[COMMAND.DOWN] = null;
                }                
                break;
        }
    }.bind(this);

    this.add_event_handlers();
}

const options = {
    enableHighAccuracy: true,
    timeout: 5000,
    maximumAge: 0,
  };

// REVIEW define a way to configure this in the UI, temporary development stuff
const ref_point = {lat: 40.43778 ,lon: 8.676421667};

// TODO
// still to review the model, so we can receive the mqtt status msgs and update the UI where necessary
var sensors = function () {

    // OBJ SETUP 
    this.config_no_connection_info = document.getElementById("config_no_connection_info");

    // scene_000, gate proximity logic
    this.scene_000_check_position = function(){
        navigator.geolocation.getCurrentPosition(this.scene_000_sucess, this.scene_000_error, options);
    }.bind(this);

    this.running_scene_000 = false;
    this.scene_000_triggered = false;
    this.dist_text = document.getElementById("text_dist");
    this.dist_unit = document.getElementById("text_unit");
    this.scene_000_sucess = function(pos){
        let point = pos.coord;
        let dist = distance_between_two_points(ref_point.lat, ref_point.lon, point.latitude, point.longitude);

        switch(dist){
            case dist <= 15:
                clearInterval(this.scene_000_check_position);
                this.scene_000_check_position = null;
                this.scene_000_triggered = false;
                break;
            case dist <= 150:
                this.scene_000_triggered = true;
                //TODO e aqui manda o comando par abrir o portão
            default:
                break;
        };

        setElementValue(this.dist_text, Math.floor((dist >= 1000)? dist / 1000: dist));
        setElementValue(this.dist_unit, (dist >= 1000)? "km":"m");
    }.bind(this);

    this.scene_000_error = function(err){
        console.warn(`ERROR(${err.code}): ${err.message}`);
    }.bind(this);

    this.proximity_change = function(evt){
        // activates/deactivates gate proximity control
        let temp_id = "scene_id_000";
        if (this.running_scene_000){
            this.elements[`${temp_id}_dis`].setAttribute("checked","");
            this.elements[`${temp_id}_ena`].removeAttribute("checked","");
            hide(this.dist_text);
            hide(this.dist_unit);
        }else{
            this.elements[`${temp_id}_ena`].setAttribute("checked","");
            this.elements[`${temp_id}_dis`].removeAttribute("checked","");
            setInterval(this.scene_000_check_position, 1);
            setElementValue(this.dist_text, "...");
            setElementValue(this.dist_unit, "...");
            show(this.dist_text);
            shoe(this.dist_unit);
        }
        this.running_scene_000 = !this.running_scene_000;
    }.bind(this);

    this.on_portao_grande = function(evt){
        // trigger gate open/close
    }

    this.on_portao_pequeno = function(evt){
        //trigger gate open (no close is possible for this gate)
    }
    // this.scene_elements = {};
    this.elements = {};
    this.running_scene_000 = false;
    this.scene_000_interval = null;
    //Scneraies Group
    // open_external_gate_in_proximity_g
    let temp_id = "scene_id_000";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.RELAY, ELEMENT_TYPE.RADIO);
    this.elements[temp_id].add_event_handlers(this.proximity_change, this.proximity_change);
    // element.sector_situation = document.getElementById(`sector_situation_enabled_${suffix}`);
    // element.data_dis = document.getElementById(`data_dis_sector_${suffix}`);
    // temp_id = "scene_id_000_dis";
    // this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.RELAY, ELEMENT_TYPE.RADIO);
    // element.data_dis.onchange = self.sector_situation_change;
    // element.data_ena = document.getElementById(`data_ena_sector_${suffix}`);
    // element.data_ena.onchange = self.sector_situation_change;
    this.elements["${temp_id}_dis"].setAttribute("checked","");

    // water_and_heat
    // all_shutters
    temp_id = "scene_id_001";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);
    // this.elements[temp_id].add_event_handlers();

    // "suite_shutters";
    temp_id = "scene_id_002";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);
    // this.elements[temp_id].add_event_handlers();
    this.guest_shutters = document.getElementById("guest_shutters");
    temp_id = "scene_id_003";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);
    // this.elements[temp_id].add_event_handlers();    
    // living_shutters
    temp_id = "scene_id_004";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.SCENE, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);
    // this.elements[temp_id].add_event_handlers();    

    // Commands Group
    // external_gate
    temp_id = "device_id_006";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.TRIGGER_SWITCH, ELEMENT_TYPE.ICON);
    this.elements[temp_id].add_event_handlers(null,this.on_portao_grande);
    // portao pequeno
    temp_id = "device_id_007";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.TRIGGER_SWITCH, ELEMENT_TYPE.ICON);
    this.elements[temp_id].add_event_handlers(null,this.on_portao_pequeno);

    // water_and_heat_l
    temp_id = "device_id_008";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.RELAY, ELEMENT_TYPE.RADIO);

    // suite_cama_shutter
    temp_id = "device_id_009";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // suite_big_shutter
    temp_id = "device_id_010";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // ana_shutter
    temp_id = "device_id_011";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // library_shutter
    temp_id = "device_id_012";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // sala_dir_shutter
    temp_id = "device_id_013";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // sala_esq_shutter
    temp_id = "device_id_014";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // sala_front_shutter
    temp_id = "device_id_015";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // st1_floor_small_shutter
    temp_id = "device_id_016";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // st1_floor_big_shutter
    temp_id = "device_id_017";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // guest_small_shutter
    temp_id = "device_id_018";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // guest_big_shutter
    temp_id = "device_id_019";
    this.elements[temp_id] = new element(temp_id, CENA_TYPE.ACTUATOR, DEVICE_TYPE.ROLLER, ELEMENT_TYPE.ICON);

    // Comsuption Group
    this.temp1 = document.getElementById("temp1");
    // Grpahics Group
    this.temp2 = document.getElementById("temp2");
    
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
        slideDown(s.panel, "setup-cenas_open");
        show(s.close_btn);
        hide(s.open_btn);
        this._openedTab = s.panel.id;
        window.localStorage.setItem('ACTIVE_ACCORDION_CENAS', this._openedTab);
    }.bind(this);

    //TESTED
    this._close_accordion = function (evt) {
        let id = evt.currentTarget.id;
        // let tgt -= id.slice(0, id.indexOf("_close"));
        let tgt = id.slice(0, id.indexOf("_close"));
        let s = this.panels[tgt];
        this._closeTab(s);
        s = null;
    }.bind(this);

    //TESTED
    this._closeTab = function (s) {
        slideUp(s.panel, "setup-cenas_open");
        hide(s.close_btn);
        show(s.open_btn);
        this._openedTab = undefined;
        window.localStorage.setItem('ACTIVE_ACCORDION_CENAS', this._openedTab);
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

    // //TESTED
    // this._hide_save_cancel_buttons = function () {
    //     hide(this.btn_cfg_cancel_changes);
    //     hide(this.btn_cfg_save_changes);
    // }.bind(this);

    // UI PREP
    //TESTED
    this.disable_screen = function () {
        let self = this;

        show(self.config_no_connection_info);

        // Scenaries Group
        disable(self.open_external_gate_in_proximity);
        disable(self.water_and_heat);
        disable(self.all_shutters);
        disable(self.suite_shutters);
        disable(self.guest_shutters);
        disable(self.living_shutters);

        // Commands Group
        disable(self.external_gate);
        disable(self.water_and_heat_l);
        disable(self.suite_cama_shutter);
        disable(self.suite_big_shutter);
        disable(self.ana_shutter);
        disable(self.library_shutter);
        disable(self.sala_dir_shutter);
        disable(self.sala_esq_shutter);
        disable(self.sala_front_shutter);
        disable(self.st1_floor_small_shutter);
        disable(self.st1_floor_big_shutter);
        disable(self.guest_small_shutter);
        disable(self.guest_big_shutter);
        // Consumption Group
        disable(self.temp1);
        // Graphics Group
        disable(self.temp2);

    }.bind(this);

    //TESTED
    this.enable_screen = function () {
        let self = this;

        hide(self.config_no_connection_info);

        //Grupo cenários
        enable(self.open_external_gate_in_proximity);
        disable(self.water_and_heat);
        enable(self.all_shutters);
        enable(self.suite_shutters);
        enable(self.guest_shutters);
        enable(self.living_shutters);

        //Grupo comandos
        enable(self.external_gate);
        enable(self.water_and_heat_l);
        enable(self.suite_cama_shutter);
        enable(self.suite_big_shutter);
        enable(self.ana_shutter);
        enable(self.library_shutter);
        enable(self.sala_dir_shutter);
        enable(self.sala_esq_shutter);
        enable(self.sala_front_shutter);
        enable(self.st1_floor_small_shutter);
        enable(self.st1_floor_big_shutter);
        enable(self.guest_small_shutter);
        enable(self.guest_big_shutter);
        //Grupo consumos
        disable(self.temp1);
        //Grupo gráficos
        disable(self.temp2);

    }.bind(this);

    // RENDER DO OBJETO PARA O ECRAN
    // //TESTED
    // this._disable_dirty_listeners = function () {
    //     if (!this._refreshing_screen_data) {
    //         this.refreshing_screen_data = true;
    //         clearListener("change", this.setup_dirty_mon);
    //     }
    // }.bind(this);

    // //TESTED
    // this._enable_dirty_listeners = function () {
    //     if (this._refreshing_screen_data) {
    //         addListener("change", this.setup_dirty_mon, this.setup_view_change);
    //         this._refreshing_screen_data = false;
    //     }
    // }.bind(this);

    // //TESTED
    // this.render_local_data = function () {
    //     let self = this;

    //     // self._disable_dirty_listeners();

    //     // self.cfg["mqtt_web_broker_address"] = MQTT.endPoint.host;
    //     // self.cfg["mqtt_web_client_id"] = MQTT.endPoint.client_id;

    //     // setElementValue(self.mqtt_web_broker_address_input, MQTT.endPoint.host);
    //     // setElementValue(self.mqtt_web_client_id_input, MQTT.endPoint.client_id);

    //     // self._enable_dirty_listeners();
    // }.bind(this);

    // TESTED
    this.render = function () {
        let self = this;
        let cfg = self.cfg;

        // self._disable_dirty_listeners();

        // self.render_local_data();

        if (!CTRL_CONN.is_everything_connected()) self.disable_screen();

        // TODO - update device status with update model data
        // //Grupo S2
        // setElementValue(self.max_sector_time_input, cfg.max_sector_time);
        // setElementValue(self.pump_recycle_time_input, cfg.pump_recycle_time);
        // setElementValue(self.schedule_def_text, (cfg.schedule_def === 1) ? "Sim" : "Não");
        // setElementValue(self.cfg_rain_amount_trigger_input, cfg.rain_alert_threshold);
        // setElementValue(self.cfg_wind_speed_trigger_input, cfg.wind_alert_threshold);
        // setElementValue(self.cfg_watering_suspend_timeout_input, cfg.watering_suspend_timeout);
        // setElementValue(self.cfg_decrease_alert_level_after_input, cfg.decrease_alert_level_after);
        // setElementValue(self.cfg_stress_control_interval, cfg.stress_control_interval);

        // //Grupo S3
        // setElementValue(self.db_maint_days_input, cfg.db_maint_days);
        // setElementValue(self.db_maint_counter_input, cfg.db_maint_counter);

        // //Grupo S4

        // //Grupo S5
        // setElementValue(self.cfg_live_since_text, cfg.live_since);
        // setElementValue(self.current_status_text, cfg.current_state);

        // self._enable_dirty_listeners();
    }.bind(this);

    // //TESTED
    // this.get_screen_data = function () {
    //     let self = this;
    //     let cfg = self.cfg;

    //     cfg.mqtt_web_broker_address = self.mqtt_web_broker_address_input.value;

    //     cfg.max_sector_time = parseFloat(self.max_sector_time_input.value);
    //     cfg.pump_recycle_time = parseFloat(self.pump_recycle_time_input.value);
    //     cfg.rain_alert_threshold = parseFloat(self.cfg_rain_amount_trigger_input.value);
    //     cfg.wind_alert_threshold = parseFloat(self.cfg_wind_speed_trigger_input.value);
    //     cfg.watering_suspend_timeout = parseFloat(self.cfg_watering_suspend_timeout_input.value);
    //     cfg.decrease_alert_level_after = parseFloat(self.cfg_decrease_alert_level_after_input.value);
    //     cfg.stress_control_interval = parseFloat(self.cfg_stress_control_interval.value);

    //     cfg.db_maint_days = parseInt(self.db_maint_days_input.value);
    //     cfg.db_maint_counter = parseInt(self.db_maint_counter_input.value);

    //     cfg.weekly_acc_precipitation = parseInt(self.precipitacao_acumulada_input.value);
    //     cfg.rain_week_acc_days = parseInt(self.rain_week_acc_days_input.value);
    //     cfg.rain_week_acc_counter = parseInt(self.rain_week_acc_counter_input.value);
    // }.bind(this);

    // eventos e listeners
    // //TESTED
    // this.setup_view_change = function setup_view_change(target) {
    //     show(this.btn_cfg_cancel_changes);
    //     show(this.btn_cfg_save_changes);
    // }.bind(this);

    // this._assign = function (fromModel, toModel) {
    //     toModel.max_sector_time = fromModel.max_sector_time;
    //     toModel.pump_recycle_time = fromModel.pump_recycle_time;
    //     toModel.rain_alert_threshold = fromModel.rain_alert_threshold;
    //     toModel.wind_alert_threshold = fromModel.wind_alert_threshold;
    //     toModel.watering_suspend_timeout = fromModel.watering_suspend_timeout;
    //     toModel.decrease_alert_level_after = fromModel.decrease_alert_level_after;
    //     toModel.stress_control_interval = fromModel.stress_control_interval;
    //     toModel.db_maint_days = fromModel.db_maint_days;
    //     toModel.db_maint_counter = fromModel.db_maint_counter;
    //     toModel.weekly_acc_precipitation = fromModel.weekly_acc_precipitation;
    //     toModel.rain_week_acc_days = fromModel.rain_week_acc_days;
    //     toModel.rain_week_acc_counter = fromModel.rain_week_acc_counter;
    //     toModel.last_change = fromModel.last_change;
    // }.bind(this);

    // TODO - terá eventos associados a cada device que vão chamar o REST API e esperar pela resposta para atualiar o estado
    //grava a informação do ecran setup
    //TESTED
    // this.btn_cfg_save_changes.onclick = function () {
    //     let self = this;
    //     self._hide_save_cancel_buttons();
    //     self.get_screen_data();
    //     self.save();

    //     try {
    //         //envia a info
    //         DB.syncToServer();
    //     } catch (err) {
    //         log.error(err);
    //     } finally {
    //         self._enable_dirty_listeners();
    //     }
    // }.bind(this);

    this.connected_event = function (param) {
        try {
            // TODO - E aqui vamos chamar o REST API para obter a informação do servidor dos devices, cenários, comandos e estado de cada coisa
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

    // TODO - o tema aqui será receber informação do MQTT com as alterações de estado dos devices que interessa
    // //TESTED
    // this.notifyNewData = function (newDataObj) {
    //     let self = this;
    //     // let cfg = self.cfg;
    //     // let newData = null;
    //     // let rendered = false;
    //     // try {
    //     //     if (newDataObj.type === "config") {
    //     //         self.newData = newDataObj.data;
    //     //         newData = self.newData;
    //     //         // CHEGOU INFO NOVA
    //     //         // VALIDAR O QUE É QUE MUDOU EM RELAÇÃO ÁS ALTERAÇÕES EFETUADAS NO ECRAN.
    //     //         // ALTERAR AS COISAS QUE NÃO FORAM ALTERADAS.
    //     //         // IDENTIFICAR AS OUTRAS E INFORMAR O UTILIZADOR - NUM CARD COM ACÇÃO PERTO DO FOOTER
    //     //         // Informação que foi alterada foi atualizada pelo servidor - quer atualizar já a informação e perder as alterações?
    //     //         // dar lista da informação que ficou diferente - para dar pista sobre o que mudou e auxiliar a decisão
    //     //         // a resposta "não" tipicamente servirá de pouco, porque nova informação chegará quando não houver alterações e esmagar a info sem avisar.
    //     //         // a não ser nos casos de acertos dos contadores....e percebi agora que isto só se verifica ai, porque o resto é mesmo configuração...
    //     //         //...check :-)  no ecran de configuração temos info de configuração :-)
    //     //         if (this._getDiff(cfg, newData, CONFIG_ATTRIBUTES_UPDATABLE, ["last_change"])) {
    //     //             //houve alterações - em tese só os contadores podem mudar - vamos ser simpáticos e dar feedback ao utilizador
    //     //             // se não responder, porque afinal não estava a olhar para o ecran, esmagamos a info.
    //     //             // neste caso esta alteração só entra em vigor após um restart.
    //     //             // REVIEW falta testar a função de restart, que em tese já funciona do lado do servidor.
    //     //             var data = {
    //     //                 message: 'Existiram alterações no servidor durante a edição.  É para esmagar as alterações?',
    //     //                 timeout: 500,
    //     //                 actionHandler: self.override_handler,
    //     //                 actionText: 'Sim!'
    //     //             };
    //     //             if (MAIN_VIEW._menu_name === "coisas-geral" && MAIN_VIEW._menu_name === "coisas-panel") {
    //     //                 //atualizamos ou não em função da resposta do user
    //     //                 MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.showSnackbar(data);
    //     //             } else {
    //     //                 //não está a acontecer nada no ecran de setup pelo que atualizamos a info
    //     //                 this.override_handler();
    //     //                 rendered = true;
    //     //             }
    //     //         } else {
    //     //             //se não houve alterações, nem render precisamos fazer
    //     //         }
    //     //         cfg.live_since = newData.live_since;
    //     //         cfg.current_state = newData.current_state;
    //     //         if (!rendered) self.render();
    //     //     }
    //     // } catch (err) {
    //     //     log.error(err);
    //     // }
    // }.bind(this);

    // //TESTED
    // this.override_handler = function (event) {
    //     let self = this;
    //     let cfg = self.cfg;
    //     let newData = self.newData;

    //     this._assign(newData, cfg);

    //     self.save();
    //     if (MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.active)
    //         MAIN_VIEW.footer_default_yes_response_snackbar.MaterialSnackbar.cleanup_();

    //     self.render();
    // }.bind(this);

    // this.cfg = model.clone();
    this.model = {};

    this._refreshing_screen_data = false;
    this.newData = null;

    this._openedTab = undefined;

    // this.setup_dirty_mon = [];
    // //local
    // this.setup_dirty_mon.push(this.mqtt_web_broker_address_input);

    // this.setup_dirty_mon.push(this.max_sector_time_input);
    // this.setup_dirty_mon.push(this.pump_recycle_time_input);
    // this.setup_dirty_mon.push(this.cfg_rain_amount_trigger_input);
    // this.setup_dirty_mon.push(this.cfg_wind_speed_trigger_input);
    // this.setup_dirty_mon.push(this.cfg_watering_suspend_timeout_input);
    // this.setup_dirty_mon.push(this.cfg_decrease_alert_level_after_input);
    // this.setup_dirty_mon.push(this.db_maint_days_input);
    // this.setup_dirty_mon.push(this.db_maint_counter_input);
    // this.setup_dirty_mon.push(this.precipitacao_acumulada_input);
    // this.setup_dirty_mon.push(this.rain_week_acc_days_input);
    // this.setup_dirty_mon.push(this.rain_week_acc_counter_input);
    // this.setup_dirty_mon.push(this.cfg_stress_control_interval);

    this.panels = {
        "cenas-group-1": {
            panel: document.getElementById("cenas-group-1"),
            accordion: document.getElementById("cenas-group-1_header"),
            open_btn: document.getElementById("cenas-group-1_open"),
            close_btn: document.getElementById("cenas-group-1_close")
        },
        "cenas-group-2": {
            panel: document.getElementById("cenas-group-2"),
            accordion: document.getElementById("cenas-group-2_header"),
            open_btn: document.getElementById("cenas-group-2_open"),
            close_btn: document.getElementById("cenas-group-2_close")
        },
        "cenas-group-3": {
            panel: document.getElementById("cenas-group-3"),
            accordion: document.getElementById("cenas-group-3_header"),
            open_btn: document.getElementById("cenas-group-3_open"),
            close_btn: document.getElementById("cenas-group-3_close")
        },
        "cenas-group-4": {
            panel: document.getElementById("cenas-group-4"),
            accordion: document.getElementById("cenas-group-4_header"),
            open_btn: document.getElementById("cenas-group-4_open"),
            close_btn: document.getElementById("cenas-group-4_close")
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

    // UI initial config
    // this._hide_save_cancel_buttons();
    // enable(this.btn_cfg_retry_broker_connection);

    // addListener("change", this.setup_dirty_mon, this.setup_view_change);

    // DB.newDataEvent.registerObserver(this, this.notifyNewData);
    CTRL_CONN.connected_event.registerObserver(this, this.connected_event);
    CTRL_CONN.disconnected_event.registerObserver(this, this.disconnected_event);
    // if (MQTT.haveController) {
    //     this.notifyConnection(true);
    // }

    this._openedTab = window.localStorage.getItem('ACTIVE_ACCORDION_CENAS');
    if (this._openedTab) {
        let s = this.panels[this._openedTab];
        this._openedTab = undefined;
        this._open_tab(s);
        s = null;
    }
};
