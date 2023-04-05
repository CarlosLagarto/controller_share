"use strict";

//TESTED
function viewCycleSector(model, mode, okCallback){

    // the window
    this.modal_Sector = document.getElementById("modalSector");
    // actions
    this.btn_save_modalSector = document.getElementById("btn_save_modalSector");
    this.btn_cancel_modalSector = document.getElementById("btn_cancel_modalSector");

    // UI
    this.modal_edit_sector = document.getElementById("modal_edit_sector");
    this.modal_view_sector = document.getElementById("modal_view_sector");

    // detail edition
    this.id = document.getElementById("sector_id");
    this.desc = document.getElementById("sector_description");
    this.description_view = document.getElementById("sector_description_view");

    this.sector_last_duration = document.getElementById("sector_last_duration");
    this.sector_last_duration_view = document.getElementById("sector_last_duration_view");

    this.max_duration =document.getElementById("sector_max_duration");
    this.max_duration_view =document.getElementById("sector_max_duration_view");
    this.deficit =document.getElementById("deficit");
    this.deficit_view =document.getElementById("deficit_view");
    this.percolation =document.getElementById("sector_precipitation");
    this.precipitation_view =document.getElementById("sector_precipitation_view");
    this.debit = document.getElementById("sector_sprinkler_debit");
    this.debit_view = document.getElementById("sector_sprinkler_debit_view");
    this.sector_situation = document.getElementById("sector_situation");
    this.sector_situation_desc = document.getElementById("sector_situation_desc");

    this.sector_status = document.getElementById("sector_status");
    this.sector_last_cycle_view = document.getElementById("sector_last_cycle_view");
    this.sector_last_start_view = document.getElementById("sector_last_start_view");
    this.sector_last_end_view = document.getElementById("sector_last_end_view");
    
    // events & listeners
    this.btn_cancel_modalSector.onclick = function() {
        hide(this.modal_Sector);
        this.change = false;
    }.bind(this);

    this.onchange = function(evt){
        this.change = true;
    }.bind(this);

    this.btn_save_modalSector.onclick = function(){
        let self = this;
        hide(self.modal_Sector);
        self.modelClone.desc = self.desc.value;
        self.modelClone.minutes_to_water = self.minutes_to_water;
        self.modelClone.max_duration = parseFloat(self.max_duration.value);
        self.modelClone.deficit= parseFloat(self.deficit.value);
        self.modelClone.percolation = parseFloat(self.percolation.value);
        self.modelClone.debit = parseFloat(self.debit.value);   
        self.modelClone.enabled = self.sector_situation.checked;
        if (this.change){
            self.modelClone.last_change = get_unix_now_time_adjusted();
            self.modelClone.OP = "U";
        }
        this.okCallback(this.modelClone)
        this.change = false;
    }.bind(this);

    this.enableChange = function(){
        let self = this;
        self.desc.onchange = self.onchange;
        self.sector_last_duration.onchange = self.onchange;
        self.max_duration.onchange = self.onchange;
        self.deficit.onchange = self.onchange;
        self.percolation.onchange = self.onchange;
        self.debit.onchange = self.onchange;
        self.sector_situation.onchange = self.onchange;        
    }.bind(this);

    this.disableChange = function(){
        let self = this;
        self.desc.onchange = null;
        self.sector_last_duration.onchange = null;
        self.max_duration.onchange = null;
        self.deficit.onchange = null;
        self.percolation.onchange = null;
        self.debit.onchange = null;
        self.sector_situation.onchange = null;
    }.bind(this);

    this.editMode = mode;  //1 = edit, 2 = view
    // work with a model copy
    this.modelClone = {};
    addMethods(this.modelClone, model);

    this.okCallback = okCallback;
    this.change = false;

    hide(this.modal_view_sector);
    hide(this.description_view);
    hide(this.sector_last_duration);
    hide(this.sector_last_duration_view);
    hide(this.max_duration_view);
    hide(this.deficit_view);
    hide(this.precipitation_view);
    hide(this.debit_view);
    hide(this.sector_situation_desc);
    hide(this.modal_edit_sector);
    hide(this.desc);
    hide(this.max_duration);
    hide(this.deficit);
    hide(this.percolation);
    hide(this.debit);
    hide(this.sector_situation);

    this.btn_save_modalSector.removeAttribute("disabled");
    this.btn_cancel_modalSector.innerHTML = "cancelar";

    enableMaterialSwitch(this.sector_situation);

    if(this.editMode === MODAL_MODE.VIEW){
        show(this.modal_view_sector);
        show(this.description_view);
        
        show(this.sector_last_duration_view);

        show(this.max_duration_view);
        show(this.deficit_view);
        show(this.precipitation_view);
        show(this.debit_view);
        show(this.sector_situation_desc);

        disableMaterialSwitch(this.sector_situation);

        this.btn_save_modalSector.setAttribute("disabled", "");
        this.btn_cancel_modalSector.innerHTML = "sair";
    }else{
        show(this.modal_edit_sector);
        show(this.desc);

        show(this.sector_last_duration);

        show(this.max_duration);
        show(this.deficit);
        show(this.percolation);
        show(this.debit);
        show(this.sector_situation);
    }

    this.render(this.modelClone);
}

viewCycleSector.prototype.render = function(model){
    let self = this;
    self.disableChange();
    setElementValue(self.id, model.id);
    setElementValue(self.desc, model.desc);
    setElementValue(self.description_view, model.desc);

    setElementValue(self.sector_last_duration, round(model.minutes_to_water, 1));
    setElementValue(self.sector_last_duration_view, round(model.minutes_to_water, 1));

    setElementValue(self.max_duration, model.max_duration);
    setElementValue(self.max_duration_view, model.max_duration);
    setElementValue(self.deficit, round(model.deficit, 0));
    setElementValue(self.deficit_view, round(model.deficit, 0));
    setElementValue(self.percolation, model.percolation);
    setElementValue(self.precipitation_view, model.percolation);
    setElementValue(self.debit, model.debit);
    setElementValue(self.debit_view, model.debit);

    if(model.enabled){
        checkMaterialSwitch(self.sector_situation);
        setElementValue(self.sector_situation_desc, "Operacional");
    }else{
        uncheckMaterialSwitch(self.sector_situation);
        setElementValue(self.sector_situation_desc, "Manutenção");
    }
    setElementValue(self.sector_status, model.status_str());
    setElementValue(self.sector_last_cycle_view, model.last_watered_in_full_str());
    setElementValue(self.sector_last_start_view, model.start_str());
    setElementValue(self.sector_last_end_view, model.end_str());
    self.enableChange();
    self = null
};

viewCycleSector.prototype.show = function(){
    show(this.modal_Sector);
};
