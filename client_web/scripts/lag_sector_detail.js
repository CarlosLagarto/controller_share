"use strict";

//TESTED
function viewCycleSector(model, mode, okCallback){

    this.editMode = mode;  //1 = edit, 2 = view
    //fazemos uma copia para podermos trabalhar á vontade e descartar no final se for caso disso
    this.modelClone = {};
    addMethods(this.modelClone, model);

    this.okCallback = okCallback;

    //a própria janela
    this.modal_Sector = document.getElementById("modalSector");
    //ações
    this.btn_save_modalSector = document.getElementById("btn_save_modalSector");
    this.btn_cancel_modalSector = document.getElementById("btn_cancel_modalSector");

    //just UI
    this.modal_edit_sector = document.getElementById("modal_edit_sector");
    this.modal_view_sector = document.getElementById("modal_view_sector");

    //navegação

    //mais isto para a edição do detalhe
    this.id = document.getElementById("sector_id");
    this.description = document.getElementById("sector_description");
    this.description_view = document.getElementById("sector_description_view");

    this.sector_last_duration = document.getElementById("sector_last_duration");
    this.sector_last_duration_view = document.getElementById("sector_last_duration_view");

    this.max_duration =document.getElementById("sector_max_duration");
    this.max_duration_view =document.getElementById("sector_max_duration_view");
    this.week_acc =document.getElementById("sector_week_acc");
    this.week_acc_view =document.getElementById("sector_week_acc_view");
    this.precipitation =document.getElementById("sector_precipitation");
    this.precipitation_view =document.getElementById("sector_precipitation_view");
    this.debit = document.getElementById("sector_sprinkler_debit");
    this.debit_view = document.getElementById("sector_sprinkler_debit_view");
    this.sector_situation = document.getElementById("sector_situation");
    this.sector_situation_desc = document.getElementById("sector_situation_desc");

    this.sector_status = document.getElementById("sector_status");
    this.sector_last_cycle_view = document.getElementById("sector_last_cycle_view");
    this.sector_last_start_view = document.getElementById("sector_last_start_view");
    this.sector_last_end_view = document.getElementById("sector_last_end_view");

    this.change = false;


    //eventos e listeners
    this.btn_cancel_modalSector.onclick = function() {
        hide(this.modal_Sector);
        this.change = false;
    }.bind(this);

    this.onchange = function(evt){
        this.change = true;
    };

    this.btn_save_modalSector.onclick = function(){
        let self = this;
        hide(self.modal_Sector);
        self.modelClone.description = self.description.value;
        self.modelClone.minutes_to_water = self.minutes_to_water;
        self.modelClone.max_duration = parseFloat(self.max_duration.value);
        self.modelClone.week_acc= parseFloat(self.week_acc.value);
        self.modelClone.precipitation = parseFloat(self.precipitation.value);
        self.modelClone.debit = parseFloat(self.debit.value);   
        self.modelClone.enabled = self.sector_situation.checked;
        if (this.change){
            self.modelClone.last_change = moment().unix();
            self.modelClone.OP = "U";
        }
        this.okCallback(this.modelClone)
        this.change = false;
    }.bind(this);

    hide(this.modal_view_sector);
    hide(this.description_view);
    hide(this.sector_last_duration);
    hide(this.sector_last_duration_view);
    hide(this.max_duration_view);
    hide(this.week_acc_view);
    hide(this.precipitation_view);
    hide(this.debit_view);
    hide(this.sector_situation_desc);
    hide(this.modal_edit_sector);
    hide(this.description);
    hide(this.max_duration);
    hide(this.week_acc);
    hide(this.precipitation);
    hide(this.debit);
    hide(this.sector_situation);

    disableModalButtons(this);

    this.sector_situation = document.getElementById("sector_situation");
    enableMaterialSwitch(this.sector_situation);

    if(this.editMode === MODAL_MODE.VIEW){
        show(this.modal_view_sector);
        show(this.description_view);
        
        show(this.sector_last_duration_view);

        show(this.max_duration_view);
        show(this.week_acc_view);
        show(this.precipitation_view);
        show(this.debit_view);
        show(this.sector_situation_desc);

        disableMaterialSwitch(this.sector_situation);

        this.btn_save_modalSector.setAttribute("disabled", "");
        this.btn_cancel_modalSector.innerHTML = "sair";
    }else{
        show(this.modal_edit_sector);
        show(this.description);

        show(this.sector_last_duration);

        show(this.max_duration);
        show(this.week_acc);
        show(this.precipitation);
        show(this.debit);
        show(this.sector_situation);
    }

    this.enableChange = function(){
        let self = this;
        self.description.onchange = self.onchange;
        self.sector_last_duration.onchange = self.onchange;
        self.max_duration.onchange = self.onchange;
        self.week_acc.onchange = self.onchange;
        self.precipitation.onchange = self.onchange;
        self.debit.onchange = self.onchange;
        self.sector_situation.onchange = self.onchange;        
    }.bind(this);

    this.disableChange = function(){
        let self = this;
        self.description.onchange = null;
        self.sector_last_duration.onchange = null;
        self.max_duration.onchange = null;
        self.week_acc.onchange = null;
        self.precipitation.onchange = null;
        self.debit.onchange = null;
        self.sector_situation.onchange = null;
    }.bind(this);

    this.render(this.modelClone);
}

viewCycleSector.prototype.render = function(model){
    let self = this;
    this.disableChange();
    setElementValue(self.id, model.id);
    setElementValue(self.description, model.description);
    setElementValue(self.description_view, model.description);

    setElementValue(self.sector_last_duration, round(model.minutes_to_water, 1));
    setElementValue(self.sector_last_duration_view, round(model.minutes_to_water, 1));

    setElementValue(self.max_duration, model.max_duration);
    setElementValue(self.max_duration_view, model.max_duration);
    setElementValue(self.week_acc, round(model.week_acc, 0));
    setElementValue(self.week_acc_view, round(model.week_acc, 0));
    setElementValue(self.precipitation, model.precipitation);
    setElementValue(self.precipitation_view, model.precipitation);
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
    setElementValue(self.sector_last_start_view, model.start_utc_ts_str());
    setElementValue(self.sector_last_end_view, model.end_utc_ts_str());
    this.enableChange();
};

viewCycleSector.prototype.show = function(){
    show(this.modal_Sector);
};
