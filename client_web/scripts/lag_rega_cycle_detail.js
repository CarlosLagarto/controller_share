"use strict";

//TESTED
function viewCycleWindow(mode, model, callback){

    this.id_cycle_edit = document.getElementById("id_cycle_edit");
    this.id_cycle_view = document.getElementById("id_cycle_view");
    this.id_cycle_error = document.getElementById("id_cycle_error");

    this.cycleEditStart_TS = document.getElementById("cycleEditStart_TS");
    this.cycleViewStart_TS = document.getElementById("cycleViewStart_TS");

    this.repeat_edit =  document.querySelectorAll("[name=repeat_edit]");
    this.repeat_view =  document.getElementById("repeat_view"); 

    this.specific_days_options =  document.querySelectorAll("[name=specific_days]");

    this.repeat_times_quantity = document.getElementById("repeat_times_quantity");
    this.repeat_times_unit =  document.querySelectorAll("[name=repeat_times_unit]");
    this.repeat_times_view = document.getElementById("repeat_times_view"); 

    this.repeatStop =  document.querySelectorAll("[name=repeatStop]");
    this.repeatStop_view =  document.getElementById("repeatStop_view");

    this.stop_date = document.getElementById("stop_date_edit");
    this.stop_date_view = document.getElementById("stop_date_view");

    this.numberOfRetries = document.getElementById("numberOfRetries");
    this.numberOfRetries_view = document.getElementById("numberOfRetries_view");

    //UX
    this.modalAddCycleDim = document.getElementById("modalAddCycleDim");
    this.modalEditCycleDim = document.getElementById("modalEditCycleDim");
    this.modalViewCycleDim = document.getElementById("modalViewCycleDim");

    this.repeat_edit_visibility = document.getElementById("repeat_edit_visibility");
    this.repeat_view_visibility = document.getElementById("repeat_view_visibility");
    //checkbox with multiple selection - always show (enabled or disabled)
    this.specific_day_visibility = document.getElementById("specific_day_visibility");

    this.stop_option = document.getElementById("stop_option");

    this.repeat_times_edit_visibility = document.getElementById("repeat_times_edit_visibility");

    this.repeatStop_edit_visibility = document.getElementById("repeatStop_edit_visibility");
    this.repeatStop_view_visibility = document.getElementById("repeatStop_view_visibility");

    this.stop_date_edit_visibility = document.getElementById("stop_date_edit_visibility");
    this.stop_date_view_visibility = document.getElementById("stop_date_view_visibility");

    this.stop_quantity_edit_visibility = document.getElementById("stop_quantity_edit_visibility");
    this.stop_quantity_view_visibility = document.getElementById("stop_quantity_view_visibility");

    this.start_ts_error = document.getElementById("start_ts_error");
    this.stop_date_error = document.getElementById("stop_date_error");

    // the window
    this.modal_Schedule = document.getElementById("modalSchedule");
    // actions
    this.btn_cancel_modalSchedule = document.getElementById("btn_cancel_modalSchedule");
    this.btn_save_modalSchedule = document.getElementById("btn_save_modalSchedule");

    this.newOrEdit = () => this.editMode === MODAL_MODE.NEW || this.editMode === MODAL_MODE.EDIT;
    this.editOrView =() => this.editMode === MODAL_MODE.EDIT || this.editMode === MODAL_MODE.VIEW;

    this.render_repeat = function(value){
        let self = this;
        switch(value){
            case SCHEDULE_REPEAT.NEVER:
                hide(this.stop_option);
                hide(self.repeat_times_edit_visibility);
                break;
            case SCHEDULE_REPEAT.SPECIFIC_WEEKDAY:
                show(self.stop_option);
                show(self.specific_day_visibility);
                if (self.newOrEdit()){
                    hide(self.repeat_times_edit_visibility);
                }else{
                    disable(self.specific_day_visibility);
                }
                break;
            case SCHEDULE_REPEAT.EVERY:
                show(self.stop_option);
                hide(self.specific_day_visibility);
                if (self.newOrEdit()){
                    show(self.repeat_times_edit_visibility);
                }
                break;
            default:
                show(self.stop_option);
                hide(self.specific_day_visibility);
                if (self.newOrEdit()){
                    hide(self.repeat_times_edit_visibility);
                }
                break;
        }
    };

    // events & listeners
    this.repeat_edit_onchange = function (item){
        this.change = true;
        this.render_repeat(item.currentTarget.value);
    }.bind(this);

    this.repeatStop_onchange = function(item){
        this.change = true;
        this.render_stop(item.currentTarget.value);
    }.bind(this);

    this.btn_cancel_modalSchedule.onclick = function() {
        this.change = false;
        hide(this.modal_Schedule);
        this.disableChange();
    }.bind(this);

    this.btn_save_modalSchedule.onclick = function(){
        // scather info to the screen (from the model)
        let validDateStartTS = true,
            validDateStopDate = true,
            validName = true,
            self = this;

        if(self.id_cycle_edit.value === CYCLE_NAME.WIZARD_NAME){
            validName = false;
        }

        if(validName){
            self.modelClone.name = self.id_cycle_edit.value;
        }else{
            show(this.id_cycle_error);
        }

        let result = validateDateField(self.cycleEditStart_TS, self.start_ts_error);
        if (result.valid){
            if (result.date in DB.cycles){
                self.start_ts_error.innerHTML = "Já existe um ciclo nesta data.";
            }else{
                self.modelClone.start = js_date_to_unix(result.date);
            }
        } else {
            validDateStartTS = false;
        }

        self.modelClone.repeat_kind = readOption(self.repeat_edit).value;

        if (self.modelClone.repeat_kind !== SCHEDULE_REPEAT.NEVER){
            self.modelClone.stop_condition = readOption(self.repeatStop).value;
            if (self.modelClone.stop_condition === SCHEDULE_STOP.RETRIES){
                self.modelClone.stop_retries = parseInt(self.numberOfRetries.value);
            } else if(self.modelClone.stop_condition === SCHEDULE_STOP.DATE){
                let result1 = validateDateField(self.stop_date, self.stop_date_error);
                if (result1.valid){
                    self.modelClone.stop_date_ts = js_date_to_unix(result1.date);
                }else{
                    validDateStopDate = false;
                }
            }
        }

        if (self.modelClone.repeat_kind === SCHEDULE_REPEAT.SPECIFIC_WEEKDAY){
            let value = 0;
            for(let i=0, len = self.specific_days_options.length; i < len; i++){
                if (self.specific_days_options[i].checked){
                    value = value | parseInt(self.specific_days_options[i].value);
                }
            }
            self.modelClone.repeat_spec_wd = value;
        } else if (self.modelClone.repeat_kind === SCHEDULE_REPEAT.EVERY){
                self.modelClone.repeat_every_qty = parseInt(self.repeat_times_quantity.value);
                self.modelClone.repeat_every_unit = readOption(self.repeat_times_unit).value;  
        }

        if (validDateStartTS && validDateStopDate){  //só fechamos a janela e chamamos o callback se tudo estiver válido
            if (DB.isCycleDefined()){
                this.start_ts_error.innerHTML = "Já existe um ciclo nesta data.";
            } else {
                hide(self.modal_Schedule);
                if (self.change){
                    self.modelClone.last_change = get_unix_now_time_adjusted();
                }
                self.disableChange();
                try{
                    self.modelClone.cycle_type = CYCLE_TYPE_INDEX.STANDARD;
                    self.modelClone.status = WATERING_STATUS.WAITING;
                    self.change = false;
                    self.okCallback(self.modelClone, self.editMode)
                }catch(e){
                    log.error(e);
                }
            }
        }
    }.bind(this);

    this.change = function(_event){
        this.change = true;
    }.bind(this);

    this.editMode = mode;  //0 = new; 1 = edit; 2 = view
    this.okCallback = callback;

    this.change = false;
    // UX
    // hide everything
    hide(this.id_cycle_view);
    hide(this.id_cycle_edit);

    hide(this.repeat_edit_visibility);
    hide(this.repeat_view_visibility );
    hide(this.specific_day_visibility );
    hide(this.repeat_times_edit_visibility );
    hide(this.repeatStop_edit_visibility);
    hide(this.repeatStop_view_visibility);
    hide(this.stop_date_edit_visibility );
    hide(this.stop_date_view_visibility );
    hide(this.stop_quantity_view_visibility );
    hide(this.stop_option );
    hide(this.modalAddCycleDim);
    hide(this.modalEditCycleDim);
    hide(this.modalViewCycleDim);

    this.btn_save_modalSchedule.removeAttribute("disabled");
    this.btn_cancel_modalSchedule.innerHTML = "cancelar";

    hide(this.cycleEditStart_TS);
    hide(this.cycleViewStart_TS);
    switch(this.editMode){
        case MODAL_MODE.NEW:
            show(this.modalAddCycleDim);
            show(this.cycleEditStart_TS);
            show(this.id_cycle_edit);
            break;
        case MODAL_MODE.EDIT:
            show(this.modalEditCycleDim);
            show(this.cycleEditStart_TS);
            show(this.id_cycle_edit);
            break;
        case MODAL_MODE.VIEW:
            show(this.modalViewCycleDim);
            show(this.cycleViewStart_TS);
            show(this.id_cycle_view);
            this.btn_save_modalSchedule.setAttribute("disabled", "");
            this.btn_cancel_modalSchedule.innerHTML = "sair";
            break;
    }

    // wotk with  model copy, so that that the original data is not changes
    this.modelClone = {};
    if (this.editOrView()){
        addMethods(this.modelClone, model);
    }else{
        addMethods(this.modelClone, new Cycle());
        this.modelClone.start = get_unix_now_time();
        this.modelClone.repeat_kind = SCHEDULE_REPEAT.NEVER;
        this.modelClone.stop_condition = SCHEDULE_STOP.NEVER;
        this.modelClone.repeat_every_qty = 1;
        this.modelClone.repeat_every_unit = SCHEDULE_REPEAT_UNIT.DAYS;
    }
    this.render(this.modelClone);
}

viewCycleWindow.prototype.disableChange = function(){
    let i = 0, len = 0;
    for (i = 0, len = this.repeat_edit.length; i < len; i = i + 1) {
        this.repeat_edit[i].onchange = null;
    }
    for (i = 0, len = this.repeatStop.length; i < len; i = i + 1) {
        this.repeatStop[i].onchange = null;
    }
    this.id_cycle_edit.onchange = null;
    this.cycleEditStart_TS.onchange = null;
    this.numberOfRetries.onchange = null;
    this.stop_date.onchange = null;
    this.specific_days_options.onchange = null;
    this.repeat_times_quantity.onchange = null;
    this.repeat_times_unit.onchange = null;
};

viewCycleWindow.prototype.enableChange = function(){
    let i=0, len=0;
    let self = this;

    self.id_cycle_edit.onchange = self.change;
    self.cycleEditStart_TS.onchange = self.change;
    self.numberOfRetries.onchange = self.change;
    self.stop_date.onchange = self.change;
    self.specific_days_options.onchange = self.change;
    self.repeat_times_quantity.onchange = self.change;
    self.repeat_times_unit.onchange = self.change;  

    if (self.editMode !== MODAL_MODE.VIEW){
        len = self.repeat_edit.length;
        for (i = 0; i < len; i = i + 1) {
            self.repeat_edit[i].onchange = self.repeat_edit_onchange
        }
        len = self.repeatStop.length;
        for (i = 0; i < len; i = i + 1) {
            self.repeatStop[i].onchange = self.repeatStop_onchange
        }
    }
};

viewCycleWindow.prototype.show = function(){
    show(this.modal_Schedule);
};

viewCycleWindow.prototype.render_stop = function(value){
    let self = this;
    hide(self.stop_date_edit_visibility);
    hide(self.stop_date_view_visibility);
    hide(self.stop_quantity_edit_visibility);
    hide(self.stop_quantity_view_visibility);
    switch(value){
        case SCHEDULE_STOP.RETRIES:
            if (self.newOrEdit()){
                show(self.stop_quantity_edit_visibility );
            }else{
                show(self.stop_quantity_view_visibility );
            }
            break;
        case SCHEDULE_STOP.DATE:
            if (self.newOrEdit()){
                show(self.stop_date_edit_visibility );
            }else{
                show(self.stop_date_view_visibility );
            }
            break;
        default:  //NEVER
            break;
    }
};

viewCycleWindow.prototype.render = function(model){
    let self = this;
    // clean screen
    self.disableChange();
    self.start_ts_error.innerHTML = "";
    self.stop_date_error.innerHTML = "";
    self.id_cycle_error.innerHTML = "";
    clearMaterialCheckboxList(self.specific_days_options);

    // scather info from model
    setElementValue(self.id_cycle_edit, model.name);
    setElementValue(self.id_cycle_view, model.name);

    setElementValue(self.cycleEditStart_TS, RFC3339DateString(unix_to_js_date_adjusted(model.start)));  
    
    setMaterialOption(self.repeat_edit, model.repeat_kind);
    let sValue = "";
    sValue = repeat_index_desc[model.repeat_kind];
    if (model.repeat_kind === SCHEDULE_REPEAT.EVERY){
        sValue = `${sValue} ${model.repeat_every_qty} ${PT_UNITS[model.repeat_every_unit]}${(model.repeat_every_qty > 1)?"s":""}`
    }
    setElementValue(self.repeat_view, sValue);

    if (model.repeat_kind === SCHEDULE_REPEAT.NEVER){
        hide(self.stop_option);
    }else{
        show(self.stop_option);
    }

    for(let i=0, len = self.specific_days_options.length; i < len; i++){
        if (parseInt(self.specific_days_options[i].value) & model.repeat_spec_wd){
            // if the curr_option bit is 1, value will be > 0
            // assumes that html value is correct
            self.specific_days_options[i].parentNode.MaterialCheckbox.check();
        }
    }

    // this only is needed with every schedule type, but in this way the code is simpler
    setElementValue(self.repeat_times_quantity, model.repeat_every_qty);
    setMaterialOption(self.repeat_times_unit, model.repeat_every_unit);

    setMaterialOption(self.repeatStop, model.stop_condition);
    setElementValue(self.repeatStop_view, stop_index_description[model.stop_condition]);

    setElementValue(self.stop_date, unix_to_js_date_adjusted(model.stop_date_ts).toString());
    setElementValue(self.stop_date_view, model.stop_date_ts_str());

    setElementValue(self.numberOfRetries, model.stop_retries);
    setElementValue(self.numberOfRetries_view, model.stop_retries + (model.stop_retries > 1)? " vezes": " vez");

    hide(self.repeat_edit_visibility);
    hide(self.repeatStop_edit_visibility);
    hide(self.repeat_view_visibility );
    hide(self.repeatStop_view_visibility);
    if (self.newOrEdit()){
        show(self.repeat_edit_visibility);
        show(self.repeatStop_edit_visibility);
    }else{
        show(self.repeat_view_visibility );
        show(self.repeatStop_view_visibility);
    }

    self.render_repeat(readOption(self.repeat_edit).value);
    self.render_stop(readOption(self.repeatStop).value);

    self.enableChange();
};
