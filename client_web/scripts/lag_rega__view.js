"use strict";

function hide_if_direct(cycle_type){
    if (cycle_type === CYCLE_TYPE_INDEX.DIRECT){
        return "hide";
    }else{
        return "";
    }
}

const getID = (originalID, prefix) => originalID.slice(prefix.length);

//TESTED
function viewRega(controller) {

    //info on top
    this.machine_status = document.getElementById("machine_status"); 
    this.infoMachineError = document.getElementById("machineError");
    this.infoMachineAlert = document.getElementById("machineAlert");
    this.alert_rain = document.getElementById("alert_rain");
    this.alert_wind = document.getElementById("alert_wind");

    this.machine_mode =  document.querySelectorAll("[name=machine_mode]");

    this.btn_aplicar_modo = document.getElementById("btn_aplicar_modo");
    this.btn_escape_confirm = document.getElementById("btn_escape_confirm");

    this.cyclesFragment = document.getElementById("cycle_list");
    this.sectorsFragment = document.getElementById("sectors_list");

    this.btn_rega_ver_ciclos = document.getElementById("btn_rega_ver_ciclos");
    this.btn_rega_ver_hist = document.getElementById("btn_rega_ver_hist");
    
    this.btn_add_cycle = document.getElementById("btn_add_cycle");
        
    this.getScoreClass = function(score){
        let _class = "js-undefined";
        if (score < 4) _class = this.scores[score];
        return _class;
    }.bind(this);

    this.render_add_cycle = function(){
        // cycles only can be defined when machine is on manual mode
        if (CTRL_REGA.is_manual() && CTRL_CONN.is_everything_connected()){
            this.btn_add_cycle.removeAttribute("disabled");
        }else{
            this.btn_add_cycle.setAttribute("disabled","");
        }
    }.bind(this);

    this.render_all = function(){
        let self = this;
        this.render_status();
        DB.get_next_cycle_id();
        self.render_cycles_V1();
        self.render_sectors_V1();
        self = null;
    }.bind(this);

    this.change_mode = function(event){
        let self = this;
        let newMode = event.currentTarget.value;
        let o = null;
        console.log("modo: " + newMode); //REVIEW

        for(let i = 0, len = self.machine_mode.length; i < len; i = i + 1){
            o = self.machine_mode[i];
            if (o.value === newMode) o.setAttribute("checked", "");
            else o.removeAttribute("checked");
        }
        show(self.btn_aplicar_modo);
        show(self.btn_escape_confirm);
        o = null;
        self = null;
    }.bind(this);

    this.enable_change_mode = function(event){
        let self = this;

        for(let i = 0, len = self.machine_mode.length; i < len; i = i + 1){
            self.machine_mode[i].removeAttribute("disabled");
        }
        self = null;
    }.bind(this);

    this.disable_change_mode = function(event){
        let self = this;

        for(let i = 0, len = self.machine_mode.length; i < len; i = i + 1){
            self.machine_mode[i].setAttribute("disabled", "");
        }
        self = null;
    }.bind(this);

    this.btn_aplicar_modo.onclick = function() {
        let self = this;
        let o = null;
        let newMode = "";
        for(let i = 0, len = this.machine_mode.length; i < len; i = i + 1){
            o = self.machine_mode[i];
            if (o.hasAttribute("checked")){
                newMode = o.value;
                break;
            }
        }
        let msg = {};
        msg["mode"] = newMode;
        buildAndSendMessage(MSG.T.CTS_STATUS_CHANGE_MODE, msg);
        hide(self.btn_aplicar_modo);
        hide(self.btn_escape_confirm);
        self.changingState = true;
        self.disable_status();
        self.machine_status.innerHTML = "a aguardar confirmação do servidor";
        addClass(self.machine_status.classList, "wait_status");
        self = null;
        o = null;
    }.bind(this);

    this.btn_escape_confirm.onclick = function() {
        let self = this;
        hide(self.btn_aplicar_modo);
        hide(self.btn_escape_confirm);
        self.changingState = false;
        self.render_status();
        self = null;
    }.bind(this);

    this.btn_add_cycle.onclick = function() {
        let self = this;
        self.cycleView = new viewCycleWindow(MODAL_MODE.NEW, self.cycleModel, self.cycleOKCallback);
        self.cycleView.show();
        self = null;
    }.bind(this);

    this.btn_rega_ver_ciclos.onclick = function() {

        if (this.btn_rega_ver_ciclos.innerHTML === "Ver Todos"){
            this.btn_rega_ver_ciclos.innerHTML = "Seguinte"
            for (let id in this.cycleElements) {
                show(this.cycleElements[id].element);
            }
            this.view_all_cycles = true;
        }
        else{
            this.btn_rega_ver_ciclos.innerHTML = "Ver Todos"
            for (let id in this.cycleElements) {
                if (id !== DB.next_cycle_id){
                    hide(this.cycleElements[id].element);
                }
            }
            this.view_all_cycles = false;
        }
    }.bind(this);

    this.btn_rega_ver_hist.onclick = function() {
        let self = this;
        self.viewCycleHistory = new viewCycleHistory();
        self.viewCycleHistory.show();
    }.bind(this);

    this.editCycle_onclick = function(event){
        let self = this;
        let ctrl = self.getRow(DB.cycles, getID(event.currentTarget.id, "edt_cycle_"));
        if (ctrl.model){
            self.cycleView = new viewCycleWindow(MODAL_MODE.EDIT, ctrl.model, self.cycleOKCallback);
            self.cycleView.show();
        }
        ctrl = null;
        self = null;
    }.bind(this);

    this.deleteElement = function(suffix){
        let element = this.cycleElements[suffix];
        delete element.work;
        delete element.wait;

        delete element.exec_cycle;
        delete element.rest_cycle;

        element.play.onclick = null;
        delete element.play;

        element.del.onclick = null;
        delete element.del;

        element.edt.onclick = null;
        delete element.edt;

        element.stop.onclick = null;
        delete element.stop;

        element.view.onclick = null;
        delete element.view;

        delete element.perc_data;
        delete element.desc_data;
        delete element.next_data;
        delete element.prev_data;
        delete element.start_data;
        delete element.end_data;
        delete element.name;
        delete element.is_direct_case;

        element = null;
    }.bind(this);

    this.addLI = function(fragment){
        let newChild = document.createElement("li");
        newChild.innerHTML = `Sem ciclos definidos! `;
        fragment.appendChild(newChild);        
        return fragment;
    }.bind(this);

    this.delCycle_onclick = function(event){
        let ctrl = this.getRow(DB.cycles, getID(event.currentTarget.id, "del_cycle_"));
        DB.delCycle(ctrl.model);

        // handle client info
        this.cyclesFragment.removeChild(this.cycleElements[ctrl.id].element);
        if (this.cyclesFragment.childNodes.length <= 0){

            let fragment = document.createDocumentFragment();
            fragment = this.addLI(fragment);

            this.cyclesFragment.innerHTML = "";
            this.cyclesFragment.appendChild(fragment);

            fragment = null;
        }
        // handle server info
        DB.syncToServer();
        ctrl = null;
    }.bind(this);

    this.viewCycle_onclick = function(event){
        let ctrl = this.getRow(DB.cycles, getID(event.currentTarget.id, "view_cycle_"));
        if (ctrl.model){
            this.cycleView = new viewCycleWindow(MODAL_MODE.VIEW, ctrl.model, this.cycleOKCallback);
            this.cycleView.show();
        }
        ctrl = null;
    }.bind(this);

    this.playCycle_onclick = function(event){
        let ctrl = this.getRow(DB.cycles, getID(event.currentTarget.id, "play_cycle_"));
        let id = ctrl.model.cycle_id;
        buildAndSendMessage(MSG.T.CTS_FORCE_CYCLE, {cycle_id : parseInt(id)});
        this.timeControler.startCycle(id);
        ctrl = null;
        id = null;
    }.bind(this);

    this.stopCycle_onclick = function(event){
        let ctrl = this.getRow(DB.cycles, getID(event.currentTarget.id, "stop_cycle_"));
        buildAndSendMessage(MSG.T.CTS_STOP_CYCLE, {cycle_id : parseInt(ctrl.model.cycle_id)});
        this.timeControler.stopCycle();
        ctrl = null;
    }.bind(this);

    this.editSector_onclick = function(event){
        let ctrl = this.getRow(DB.sectors, getID(event.currentTarget.id, "edt_sector_"));
        if (ctrl.model){
            this.sectorView = new viewCycleSector(ctrl.model, MODAL_MODE.EDIT, this.sectorOKCallback);
            this.sectorView.show();
        }
        ctrl = null;
    }.bind(this);

    this.playSector_onclick = function(event){
        let ctrl = this.getRow(DB.sectors, getID(event.currentTarget.id, "play_sector_"));
        buildAndSendMessage(MSG.T.CTS_FORCE_SECTOR, {running_ptr: {sec_id : parseInt(ctrl.id)}});
        this.timeControler.startSector(ctrl.id);
        ctrl = null;
    }.bind(this);

    this.viewSector_onclick = function(event){
        let ctrl = this.getRow(DB.sectors, getID(event.currentTarget.id, "view_sector_"));
        if (ctrl.model){
            this.sectorView = new viewCycleSector(ctrl.model, MODAL_MODE.VIEW, this.sectorOKCallback);
            this.sectorView.show();
        }
        ctrl = null;
    }.bind(this);

    this.stopSector_onclick = function(event){
        // DESIGN NOTE 
        // In direct mode when running a cucle, stopping the running sector stops de cycle.
        // This means that when a cycle is launched either one waits for this ending, or starts over from the beginning.
        // The alternative of stepping to the next sector automatically also seemed unatural to me
        // This means that either we are  testing or doing something in the water system or watering manually individual sectors
        // and so I decided that it only makes sense to direct water a sector individually to do that
        let cycle = null;
        let self = this;
        for (let id in DB.cycles){
            cycle = DB.cycles[id];
            // procuramos o ciclo directo que esteja ativo 
            if (cycle.cycle_type === CYCLE_TYPE_INDEX.DIRECT && cycle.is_running()){
                let cycle = DB.cycles[id];
                let clickEvent = new MouseEvent("click", {
                    bubbles: true,
                    cancelable: true,
                  });
                let cycleElement = self.cycleElements[cycle.cycle_id];
                // simualate the cycle click event
                cycleElement.stop.dispatchEvent(clickEvent);
                break;
            }
        }
        self = null;
        cycle = null;

    }.bind(this);

    this.sector_situation_change = function(event){
        let ctrl = this.getRow(DB.sectors, getID(event.currentTarget.name, "sector_situation_"));

        ctrl.model.enabled = event.currentTarget.id.indexOf("data_dis") <= -1;
        ctrl.model.last_change = get_unix_now_time_adjusted();
        DB.updateSector(ctrl.model);
        DB.syncToServer();
        this.render_sectors_V1();
        ctrl = null;
    }.bind(this);

    this.cycleOKCallback = function(newModel, editMode){
        try{
            if (editMode === MODAL_MODE.NEW){
                DB.addCycle(newModel);
            }else{
                DB.updateCycle(newModel);
            }
            this.render_cycles_V1(DB.cycles);
            DB.syncToServer()
        }catch(e){
            log.error(e);
        }
    }.bind(this);

    this.disable_status = function(){
        for(let i = 0, len = this.machine_mode.length; i < len; i = i + 1){
            this.machine_mode[i].setAttribute("disabled","");
        }    
    }.bind(this);

    this.enable_status = function(){
        for(let i = 0, len = this.machine_mode.length; i < len; i = i + 1){
            this.machine_mode[i].removeAttribute("disabled");
        }
        removeClass(this.machine_status.classList, "wait_status");
    }.bind(this);

    this.setAlertVisibility =  function (alert_type){
        let self = this;
        let show = false;
        if (alert_type === ALERT_TYPE.RAIN){
            show = true;
            show(self.alert_rain)
        }
        if (alert_type === ALERT_TYPE.WIND){
            show = true;
            show(self.alert_wind)
        }
        if (show) {
            show(self.infoMachineAlert);
        }else{
            hide(self.infoMachineAlert);
        }
    }.bind(this);

    this.render_status = function(){
        let self = this; 
        if(!self.changingState){

            setElementValue(self.machine_status, WATER_MACHINE_STATUS[self.controller.model.machine_status]);  

            if ([WATER_MACHINE_MODE.MANUAL, WATER_MACHINE_MODE.STANDARD, WATER_MACHINE_MODE.WIZARD].includes(self.controller.model.mode)) {
                setStandardOption(self.machine_mode, self.controller.model.mode);
                if (CTRL_CONN.is_everything_connected()){
                    self.enable_status();
                }else{
                    self.disable_status();
                }
            }
            // when updateing the status, any previous action of mode changing is ignored, and we hide the button
            // this may cause a running condition, but in thesis (have to test...) only when starting the application
            if (isVisible(self.btn_aplicar_modo)){
                MAIN_VIEW.StatusMsg.setTemp("Chegou uma atualização do servidor que anulou o modo seleccionado.", 10);
            }
        } 

        setVisibility(self.infoMachineError, self.controller.model.error);
        self.setAlertVisibility(self.controller.model.alert);

        hide(self.btn_aplicar_modo);
        hide(self.btn_escape_confirm);
        self = null;
    }.bind(this);

    this._commonCleanListeners = function(element){
        element.play.onclick = null;
        element.edt.onclick = null;
        element.stop.onclick = null;
        element.view.onclick = null;
    }.bind(this);

    this.cleanCycleListeners = function(cycle_element){
        cycle_element.del.onclick = null;
        this._commonCleanListeners(cycle_element);
    }.bind(this);

    this.addCycleUIElements = function(suffix){
        let self = this;
        if (suffix in self.cycleElements) {
            let element = self.cycleElements[suffix];
            element.work = document.getElementById(`work_cycle_${suffix}`);
            element.wait = document.getElementById(`wait_cycle_${suffix}`);

            element.exec_cycle = document.getElementById(`exec_cycle_${suffix}`);
            element.rest_cycle = document.getElementById(`rest_cycle_${suffix}`);

            element.play = document.getElementById(`play_cycle_${suffix}`);
            element.play.onclick = self.playCycle_onclick;

            element.del = document.getElementById(`del_cycle_${suffix}`);
            element.del.onclick = self.delCycle_onclick;

            element.edt = document.getElementById(`edt_cycle_${suffix}`);
            element.edt.onclick = self.editCycle_onclick;

            element.stop = document.getElementById(`stop_cycle_${suffix}`);
            element.stop.onclick = self.stopCycle_onclick;

            element.view = document.getElementById(`view_cycle_${suffix}`);
            element.view.onclick = self.viewCycle_onclick;

            element.perc_data = document.getElementById(`perc_data_cycle_${suffix}`);
            element.desc_data = document.getElementById(`desc_data_cycle_${suffix}`);
            element.next_data = document.getElementById(`next_data_cycle_${suffix}`);
            element.prev_data = document.getElementById(`prev_data_cycle_${suffix}`);
            element.start_data = document.getElementById(`start_data_cycle_${suffix}`);
            element.end_data = document.getElementById(`end_data_cycle_${suffix}`);        
            element.name_data = document.getElementById(`name_data_cycle_${suffix}`)
            element.is_direct_case = document.getElementById(`is_direct_case_${suffix}`)
            element = null;
        }
        else{
            log.error("sufixo desconhecido: " + suffix);
        }
        self = null;
    }.bind(this);

    this.addCycleUI = function(cycle, fragment){
        let self = this;
        let newChild = document.createElement("li");
        addClass(newChild.classList, "cycle");
        let suffix = `${cycle.cycle_id}`;
        newChild.id = `${suffix}`;
        newChild.innerHTML = `
<div style="min-width:6.4vw">
    <svg id="stop_cycle_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
        <use xlink:href="#stop-thing"/>
    </svg>
    <svg id="play_cycle_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
        <use xlink:href="#start-thing"/>
    </svg>
    <svg id="work_cycle_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
        <use xlink:href="#working-thing"/>
    </svg>
    <svg id="wait_cycle_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
        <use xlink:href="#waiting-thing"/>
    </svg>
</div>
<div style="display: flex; flex-direction: column;margin-left: 0.2rem;width:100%">
    <div class="label-value-pair">
        <span class="col-label-sm1">Nome:</span>
        <span id="name_data_cycle_${suffix}" class="col-value-lg">${cycle.name_str()}</span>
    </div>
    <div id="rest_cycle_${suffix}" class="cycle_sector_flow" >
        <div style="width: 100%;">
            <div class="label-value-pair ${hide_if_direct(cycle.cycle_type)}">
                <span class="col-label-sm1">Programa:</span>
                <span id="desc_data_cycle_${suffix}">${cycle.desc()}</span>
            </div>
            <div class="label-value-pair">
                <span class="col-label-sm1">Regou:</span>
                <span id="prev_data_cycle_${suffix}" style="width: max-content;">${cycle.prev_cycle_description()}</span>
                <span style="width: 5px;"></span>
            </div>
            <div class="label-value-pair" id="is_direct_case_${suffix}">
                <span class="col-label-sm1">Rega:</span>
                <span id="next_data_cycle_${suffix}">${cycle.next_cycle_description()}</span>
                <span style="width: 5px;"></span>
            </div>
        </div>
        <div class="margin-right-last-col">
            <svg id="del_cycle_${suffix}" viewBox="0 0 24 24" class="img-body hide">
                <use xlink:href="#cancel"/>
            </svg>
            <svg id="edt_cycle_${suffix}" viewBox="0 0 24 24" class="img-body hide">
                <use xlink:href="#edit"/>
            </svg>
            <svg id="view_cycle_${suffix}" viewBox="0 0 24 24" class="img-body hide">
                <use xlink:href="#view"/>
            </svg>
        </div>
    </div>
    <div id="exec_cycle_${suffix}" class="executing hide" >
        <div id="start_data_cycle_${suffix}" class="executing__first-label" style="margin-left: 0.2rem;">${cycle.start_str()}</div>
        <div class="ProgressBar">
            <div class="ProgressBar-background"></div>
            <div id="perc_data_cycle_${suffix}" class="ProgressBar-percentage" style="width: ${cycle.start_exec_perc_str()}"></div>
        </div>
        <div id="end_data_cycle_${suffix}" class="executing__last-label">${cycle.end_str()}</div>
    </div>
</div>`;
        fragment.appendChild(newChild);

        self.cycleElements[suffix] = {};
        self.cycleElements[suffix].element = newChild;

        self = null;
        return suffix;
    }.bind(this);

    this.cyclesBuild = function(){
        // build cycles UI
        let fragment = document.createDocumentFragment();
        let self = this; 

        // clean listeners
        for (let element in self.cycleElements) {
            self.cleanCycleListeners(self.cycleElements[element]);
        }
        self.cycleElements = {};  // clean all elements
        self.cyclesFragment.innerHTML = "";

        let cycle = null;
        for (let id in DB.cycles){
            cycle = DB.cycles[id];

            if (cycle.op !== OP.D){ // ignore deleted cycles still not synced with the backend
                self.noCycles = false;
                self.addCycleUI(cycle, fragment);
            }
            cycle = null;
        }

        if (self.noCycles){
            fragment = this.addLI(fragment);
        }

        // add cycle LI
        self.cyclesFragment.appendChild(fragment);

        for(let suffix in self.cycleElements){
            self.addCycleUIElements(suffix);     
        }
        fragment = null;
        self = null;
    }.bind(this);

    this._nullifyCommonElements = function(element){
        element.work = null;
        element.wait = null;

        element.play = null;
        element.edt = null;
        element.stop = null;
        element.view = null;
    }.bind(this);

    this._hideCommonElements = function (element) {
        hide(element.play);
        hide(element.wait);
        hide(element.stop);
        hide(element.work);
        hide(element.edt);
        hide(element.view);
    }.bind(this);

    this.delete_cycle_from_view = function(cycle_id){
        let self = this;
        let cycleElement = null;
        if (cycle_id in self.cycleElements){
            cycleElement = self.cycleElements[cycle_id];
            // stil to understand if the DOM have internal pointers.  Just in case, lets help the machine, memory wise and clean this stuff
            self.cleanCycleListeners(cycleElement);
            if (self.cyclesFragment.contains(cycleElement.element)){
                self.cyclesFragment.removeChild(cycleElement.element);
            }
            // help GC.  Still to understand GC internal algorithm
            // but have been reading some complaints about circular references and dangling pointers in DOM
            self._nullifyCommonElements(cycleElement);

            cycleElement.exec_cycle = null;
            cycleElement.rest_cycle = null;
            cycleElement.del = null;
            cycleElement.perc_data = null;
            cycleElement.desc_data = null;
            cycleElement.next_data = null;
            cycleElement.start_data = null;
            cycleElement.end_data = null;
            cycleElement.is_direct_case = null;

            delete self.cycleElements[cycle_id];
        }
        cycleElement = null;
        self = null;
    }

    this.render_cycles_V1 = function(){
        let model = DB.cycles;
        let fragment = document.createDocumentFragment();
        let self = this;
        let count = 0;
        let cycle = null;
        let suffix = "";
        let cycleElement = null;

        self.render_add_cycle();

        for (let id in self.cycleElements){
            if (!(id in model)){
                self.delete_cycle_from_view(id)
            }
        }
        for(let id in model){
            cycle = model[id];
            if (cycle.op !== OP.D){
                if (!(cycle.cycle_id in self.cycleElements)){
                    if (self.noCycles){
                        self.cyclesFragment.innerHTML = "";
                    }
                    suffix = self.addCycleUI(cycle, fragment);
                    self.cyclesFragment.appendChild(fragment);
                    self.addCycleUIElements(suffix);
                    self.noCycles = false;
                }
                count += 1;
                cycleElement = self.cycleElements[cycle.cycle_id];
                // handle shows/hides & updates
                this._hideCommonElements(cycleElement);

                hide(cycleElement.exec_cycle);
                hide(cycleElement.rest_cycle);

                cycleElement.perc_data.style["width"] = `${cycle.start_exec_perc_str()}%`;
                cycleElement.start_data.innerHTML = cycle.start_str();
                cycleElement.end_data.innerHTML = cycle.end_str();
                cycleElement.next_data.innerHTML = cycle.start_ts_str();
                cycleElement.prev_data.innerHTML = cycle.prev_cycle_description();
                cycleElement.desc_data.innerHTML = cycle.desc();
                cycleElement.name_data.innerHTML = cycle.name_str(self.controller.model.machine_status);

                // should we show only the next running cycle, or all of them
                if (cycle.cycle_id.toString() !== DB.next_cycle_id && !this.view_all_cycles){
                    hide(cycleElement.element);
                }else{
                    show(cycleElement.element);
                }

                if (cycle.is_running() && self.controller.model.machine_status !== "ManWtrSectorDirect"){
                    show(cycleElement.exec_cycle);
                    // we can always stop regardless of cycle type
                    // TODO if not manual, one should change the machine to manual mode.  
                    show(cycleElement.stop);
                }else{
                    show(cycleElement.rest_cycle);

                    if(CTRL_REGA.is_manual()){
                        addClass(cycleElement.rest_cycle.classList,"disabled-text");
                    }else{
                        removeClass(cycleElement.rest_cycle.classList,"disabled-text");
                        show(cycleElement.wait);
                    }
                }
                
                if (cycle.cycle_type === CYCLE_TYPE_INDEX.DIRECT){
                    hide(cycleElement.is_direct_case);
                    hide(cycleElement.del);
                    hide(cycleElement.edt);
                    if (!cycle.is_running() && CTRL_CONN.is_everything_connected() && CTRL_REGA.is_manual()){
                        show(cycleElement.play)
                    }
                    if (CTRL_REGA.is_manual()){
                        hide(cycleElement.play);
                        hide(cycleElement.wait);
                    }
                }else{
                    show(cycleElement.is_direct_case);
                }
                
                if (cycle.cycle_type === CYCLE_TYPE_INDEX.STANDARD){
                    if (!cycle.is_running()){
                        show(cycleElement.del);
                        show(cycleElement.edt);
                    }else{
                        hide(cycleElement.del);
                        hide(cycleElement.edt);    
                    }
                }
                if (cycle.cycle_type === CYCLE_TYPE_INDEX.WIZARD){
                    hide(cycleElement.del);
                    hide(cycleElement.edt);
                    hide(cycleElement.play)
                    show(cycleElement.view);                    
                }

                if (cycle.cycle_type === CYCLE_TYPE_INDEX.COMPENSATION){
                    hide(cycleElement.del);
                    hide(cycleElement.edt);
                    hide(cycleElement.view);
                }
            }else{ 
                self.delete_cycle_from_view(cycle_id);
            }
        }
        if (count === 0 ){
            self.noCycles = true;
            self.cyclesFragment.innerHTML = "";
            fragment = this.addLI(fragment);
            self.cyclesFragment.appendChild(fragment);
        }

        model = null;
        fragment = null;
        count = null;
        cycle = null;
        suffix = null;
        cycleElement = null;
        self = null;
    }.bind(this);

    this.addSectorUIElements = function(suffix){
        let self = this;
        let element = self.sectorElements[suffix];
        element.work = document.getElementById(`work_sector_${suffix}`);
        element.wait = document.getElementById(`wait_sector_${suffix}`);

        element.water_status = document.getElementById(`water_status_sector_${suffix}`);
        element.score = document.getElementById(`score_sector_${suffix}`);
        element.exec = document.getElementById(`exec_sector_${suffix}`);

        element.play = document.getElementById(`play_sector_${suffix}`);
        element.play.onclick = self.playSector_onclick;

        element.edt = document.getElementById(`edt_sector_${suffix}`);
        element.edt.onclick = self.editSector_onclick;

        element.stop = document.getElementById(`stop_sector_${suffix}`);
        element.stop.onclick = self.stopSector_onclick;

        element.view = document.getElementById(`view_sector_${suffix}`);
        element.view.onclick = self.viewSector_onclick;

        element.data_water_num = document.getElementById(`data_water_num_sector_${suffix}`);
        element.data_water_perc = document.getElementById(`data_water_perc_sector_${suffix}`);

        element.data_desc = document.getElementById(`data_desc_sector_${suffix}`);//
        element.next_start_desc = document.getElementById(`next_start_desc_sector_${suffix}`);

        element.data_start_desc = document.getElementById(`data_start_desc_sector_${suffix}`);
        element.data_start = document.getElementById(`data_start_sector_${suffix}`); 
        element.data_progress_perc = document.getElementById(`data_progress_perc_sector_${suffix}`);
        element.data_end = document.getElementById(`data_end_sector_${suffix}`);
        element.sector_situation = document.getElementById(`sector_situation_enabled_${suffix}`);
        element.data_dis = document.getElementById(`data_dis_sector_${suffix}`);
        element.data_dis.onchange = self.sector_situation_change;
        element.data_ena = document.getElementById(`data_ena_sector_${suffix}`);
        element.data_ena.onchange = self.sector_situation_change;

        self = null;
        element = null;
    }.bind(this);

    this.addSectorUI = function(sector, fragment){
        let self = this;
        let newChild = document.createElement("li");
        addClass(newChild.classList, "sector");
        let suffix = `${sector.id}`;
        newChild.id = `${suffix}`;
        newChild.innerHTML = `
<div class="sector-status">
    <div id="water_status_sector_${suffix}" class="box hide">
        <div class="percent">
            <div id="data_water_num_sector_${suffix}" class="percentNum" >${sector.watering_percent_str()}"</div>
            <div class="percentB">%</div>
        </div>
        <div id="data_water_perc_sector_${suffix}" class="water" style="transform: translate(0, ${(1 - sector.watering_percent) * 100}%)";>
            <svg viewBox="0 0 560 20" class="water_wave water_wave_back">
                <use xlink:href="#wave"></use>
            </svg>
            <svg viewBox="0 0 560 20" class="water_wave water_wave_front">
                <use xlink:href="#wave"></use>
            </svg>
        </div>
    </div>
    <div id="score_sector_${suffix}" class="score-sector">${sector.stress_perc_str()}%</div>
</div>
<div class="sector-info">
    <div>
        <svg id="play_sector_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
            <use xlink:href="#start-thing"/>
        </svg>
        <svg id="stop_sector_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
            <use xlink:href="#stop-thing"/>
        </svg>
        <svg id="wait_sector_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical">
            <use xlink:href="#waiting-thing"/>
        </svg>
        <svg id="work_sector_${suffix}" viewBox="0 0 24 24" class="img-body center-vertical hide">
            <use xlink:href="#working-thing"/>
        </svg>
    </div>
    <div class="cycle_sector_flow" >
        <div class="sector-info__info">
            <div class="label-value-pair sector-info__adjust-bottom">
                <span id="data_desc_sector_${suffix}" class="col-label-st">${sector.name}</span>
            </div>
            <div id="next_start_desc_sector_${suffix}" class="label-value-pair hide">
                <span class="col-label-st">Vai regar:</span>
                <span id="data_start_desc_sector_${suffix}" class="col-value-sm">${sector.start_str()}</span>
            </div>
            <div id="exec_sector_${suffix}" class="sector-info__executing hide" >
                <div id="data_start_sector_${suffix}">${sector.start_str()}</div>
                <div class="ProgressBar ProgressBar__sector-adjust">
                    <div class="ProgressBar-background"></div>
                    <div id="data_progress_perc_sector_${suffix}" class="ProgressBar-percentage" style="width: ${sector.watering_percent_str()}%"></div>
                </div>
                <div id="data_end_sector_${suffix}" class="sector-info__executing__last-col-adjust">${sector.end_hour_ts_str()}</div>
            </div>
        </div>
        <div class="margin-right-last-col">
            <div class="sector-info__status">
                <div id="sector_situation_enabled_${suffix}" class="radioGroup1 hide">
                    <label>
                        <input id="data_dis_sector_${suffix}" type="radio" name="sector_situation_${suffix}" value="0" ${(!sector.enabled) ? "checked" : ""}>
                        <span>Manutenção</span>
                    </label>
                    <label>
                        <input id="data_ena_sector_${suffix}" type="radio" name="sector_situation_${suffix}" value="1" ${(sector.enabled) ? "checked" : ""}>
                        <span>Operacional</span>
                    </label>
                </div>
            </div>
            <div class="sector-info__buttons">
                <div class="full-width_helper"></div>
                <div>
                    <svg id="edt_sector_${suffix}" viewBox="0 0 24 24" class="img-body hide">
                        <use xlink:href="#edit"/>
                    </svg>
                    <svg id="view_sector_${suffix}" viewBox="0 0 24 24" class="img-body hide">
                        <use xlink:href="#view"/>
                    </svg>
                </div>
            </div>
        </div>
    </div>
</div>`;
        fragment.appendChild(newChild);

        self.sectorElements[suffix] = {};
        self.sectorElements[suffix].element = newChild;

        newChild = null;
        suffix = null;
        self = null;

        return suffix;
    }.bind(this);

    this.cleanSectorListeners = function(sector_element){
        this._commonCleanListeners(sector_element);
        sector_element.data_dis.onchange = null;
        sector_element.data_ena.onchange = null;
    }.bind(this);

    this.sectorsBuild = function(){
        // build sector UI
        let fragment = document.createDocumentFragment();
        let self = this; 
        let newChild = null;

        //clean listeners
        for (let element in self.sectorElements) {
            self.cleanSectorListeners(self.sectorElements[element]);
        }
        self.sectorElements = {};  // clean all elements
        self.sectorsFragment.innerHTML = "";
        let sector = null;
        for (let id in DB.sectors){
            sector = DB.sectors[id];

            if (sector.op !== OP.D){ // ignore deleted records not synced wioth the backend
                self.noSectors = false;
                self.addSectorUI(sector, fragment);
            }else{
                if (!(id in self.sectorElements)){
                    delete self.sectorElements[id];
                }
            }
            sector = null;
        }

        if (self.noSectors){
            newChild = document.createElement("li");
            newChild.innerHTML = "Sem setores! A aguardar pelo servidor.";
            fragment.appendChild(newChild);
        }

        //add sector UI
        self.sectorsFragment.appendChild(fragment);

        for(let suffix in self.sectorElements){
            self.addSectorUIElements(suffix);     
        }
        fragment = null;
        newChild = null;
        self = null;
    }.bind(this);

    this.render_sectors_V1 = function(){
        let fragment = document.createDocumentFragment();
        let self = this;
        let count = 0;
        let sector = null;
        let suffix = "";
        let sectorElement = null;
        let model = DB.sectors;

        for(let id in model){
            sector = model[id];
            if (sector.op !== OP.D){
                if (!(sector.id in self.sectorElements)){
                    if (self.noSectors){
                        self.sectorsFragment.innerHTML = "";
                    }
                    suffix = self.addSectorUI(sector, fragment);
                    self.sectorsFragment.appendChild(fragment);
                    self.addSectorUIElements(suffix);
                    self.noSectors = false;
                }
                count += 1;
                sectorElement = self.sectorElements[sector.id];
                // shows/hides & updates
                this._hideCommonElements(sectorElement);

                hide(sectorElement.exec);
                hide(sectorElement.water_status);
                hide(sectorElement.score);

                hide(sectorElement.next_start_desc);
                hide(sectorElement.sector_situation);

                // update html info - One should update only the visible elements...with more code complexity.  decided that the performance penalty is negligible
                sectorElement.data_water_num.innerHTML = round(sector.watering_percent, 0);
                sectorElement.data_water_perc.style["transform"] = `translate(0, ${100. - sector.watering_percent}%`;

                sectorElement.score.inneHTML = `${sector.stress_perc_str()}%`;
                for (let score in this.scores){
                    removeClass(sectorElement.score.classList, score);
                }
                addClass(sectorElement.score.classList, self.getScoreClass(sector.stress_score));

                sectorElement.data_start.innerHTML = sector.start_str();
                sectorElement.data_progress_perc.style["width"] = `${sector.watering_percent_str()}%`;
                sectorElement.data_end.innerHTML = sector.end_hour_ts_str();

                sectorElement.data_start_desc.innerHTML = sector.start_str();
                sectorElement.data_desc.innerHTML = sector.name;

                if (sector.enabled){
                    sectorElement.data_dis.removeAttribute("checked", "");
                    sectorElement.data_ena.setAttribute("checked","");
                }else{
                    sectorElement.data_dis.setAttribute("checked", "");
                    sectorElement.data_ena.removeAttribute("checked","");
                }
                // show what is needed
                if (CTRL_CONN.is_everything_connected()){
                    if (sector.is_running()){
                        show(sectorElement.exec);
                        show(sectorElement.water_status);
                        show(sectorElement.view);

                        if(CTRL_REGA.is_manual()){
                            show(sectorElement.stop);
                        }else{
                            show(sectorElement.work);
                        }
                    }else{
                        if(CTRL_REGA.is_manual()){
                            show(sectorElement.play);
                            if (!sector.enabled || (sector.is_waiting() && this.any_cycle_running())){
                                addClass(sectorElement.play.classList, "dim-svg");
                                sectorElement.play.onclick = null;
                            }else{
                                removeClass(sectorElement.wait.classList, "dim-svg");
                                if (!sectorElement.play.onclick){
                                    sectorElement.play.onclick = self.playSector_onclick;
                                }
                            }
                            hide(sectorElement.next_start_desc);
                            show(sectorElement.edt);
                            sectorElement.sector_situation.removeAttribute("disabled", "");
                        }else{
                            removeClass(sectorElement.wait.classList, "dim-svg");
                            show(sectorElement.wait);
                            show(sectorElement.view);
                            sectorElement.sector_situation.setAttribute("disabled", "");
                            show(sectorElement.next_start_desc);
                        }
                        show(sectorElement.sector_situation);
                        show(sectorElement.score);
                    }
                } else {
                    show(sectorElement.sector_situation);
                    show(sectorElement.score);
                    show(sectorElement.wait);
                    addClass(sectorElement.wait.classList, "dim-svg");
                    show(sectorElement.view);
                    sectorElement.sector_situation.setAttribute("disabled", "");
                }
            }else if(sector.id in self.sectorElements){
                sectorElement = self.sectorElements[sector.id];
                this.cleanCycleListeners(sectorElement);  // see cycle handling comments
                self.sectorsFragment.removeChild(sectorElement.element);

                // see cycle handling comments
                this._nullifyCommonElements(sectorElement);

                sectorElement.exec = null;
                sectorElement.water_status = null;
                sectorElement.score = null;
                sectorElement.sector_situation = null;
                sectorElement.data_water_num = null;
                sectorElement.data_water_perc = null;
                sectorElement.data_start = null;
                sectorElement.data_progress_perc = null;
                sectorElement.data_end = null;

                sectorElement.data_start_desc = null;
                sectorElement.next_start_desc = null;
                sectorElement.data_desc = null;

                sectorElement.data_dis = null;
                sectorElement.data_ena = null;

                delete self.sectorElements[sector.id];
            }
        }
        if (count === 0 ){
            self.noSectors = true;
            self.sectorsFragment.innerHTML = "";
            let newChild = document.createElement("li");
            newChild.innerHTML = `Sem ciclos definidos!`;
            fragment.appendChild(newChild);            
            newChild = null;
        }

        fragment = null;
        count = null;
        sector = null;
        suffix = null;
        sectorElement = null;
        model = null;
        self = null;
    }.bind(this);

    //TESTED
    this.sectorOKCallback = function(newModel){
        try{
            DB.updateSector(newModel);
            this.render_sectors_V1(DB.sectors);   
            DB.syncToServer()
        }catch(e){
            log.error(e);
        }
    }.bind(this);

    //TESTED
    this.getRow = function(table, id){
        let ctrl = {length: Object.keys(table).length, model : null, id : ""};
        if(id in table){
            ctrl.model = table[id];
            ctrl.id = id;
        }
        return ctrl;
    }.bind(this);

    this.check_running_things = function(){
        // for when refreshing the page during a running cycle or sector
        let cycle_running = false;
        for(let cycleId in DB.cycles){
            if (DB.cycles[cycleId].is_running()){
                this.timeControler.startCycle(cycleId);
                cycle_running = true;
            }
        }
        if (!cycle_running){
            // lets find out if we are in manual mode
            if (DB.config["mode"] === WATER_MACHINE_MODE.MANUAL){
                for(let sectorId in DB.sectors){
                    if(DB.sectors[sectorId].is_running()){
                        this.timeControler.startSector(sectorId);
                    }
                }
            }
        }
        cycle_running = null;
    }.bind(this);

    this.any_cycle_running = () => this.timeControler.cycleId !== -1;

    // setp properties
    this.cycleElements = {};
    this.sectorElements = {};

    this.controller = controller;

    this.changingState = false;
    this.noCycles = true;
    this.noSectors = true;

    this.view_all_cycles = false;

    this.timeControler = new time_controller(DB.config.pump_recycle_time, this.render_all);

    let element = null;
    for(let i = 0, len = this.machine_mode.length; i < len; i = i + 1){
        element = this.machine_mode[i];
        element.onclick = this.change_mode;
    }
    element = null;

    this.scores = ["js-over-irrigated", "js-normal", "js-alert", "js-emergency"];
}
