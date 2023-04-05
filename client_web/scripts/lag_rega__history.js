"use strict";

function get_str_hour_and_minute(date){
    let str_hours = "";
    if (date.getUTCHours() < 10) {
        str_hours = "0" + date.getUTCHours();
    }else{
        str_hours = date.getUTCHours();
    }
    let str_minutes = "";
    if (date.getUTCMinutes() < 10){
        str_minutes = "0" + date.getUTCMinutes();
    }else{
        str_minutes = date.getUTCMinutes();
    }
    return str_hours + ":" + str_minutes;
}

function WateredCycle (cycle){
    this.data = cycle;
    this.start = function(){
        let date = unix_to_js_date_adjusted(this.data.start);
        let str_month = MONTH_NAME[date.getUTCMonth()];

        let str_hours_and_minutes = get_str_hour_and_minute(date);

        // strangely javascript defines getUTCDay as the day of the week from 0 a 6...
        let str_start = str_month + "/" + date.getUTCDate() + " " + str_hours_and_minutes;
        return str_start;
    }.bind(this);

    this.end = function(){
        let date = unix_to_js_date_adjusted(this.data.end);
        let str_hours_and_minutes = get_str_hour_and_minute(date);
        return str_hours_and_minutes;
    }.bind(this);
}

function WateredSector(sector){
    this.data = sector;

    this.start = function(){
        let date = unix_to_js_date_adjusted(this.data.start);
        let str_hours_and_minutes = get_str_hour_and_minute(date);
        return str_hours_and_minutes;
    }.bind(this);

    this.end = function(){
        let date = unix_to_js_date_adjusted(this.data.end);
        let str_hours_and_minutes = get_str_hour_and_minute(date);
        return str_hours_and_minutes;
    }.bind(this);

    this.duracao = function(){
        let str_dur = this.data.minutes_to_water_acc.toFixed(1);
        if (this.data.minutes_to_water_acc < 10){
            str_dur = " " + str_dur;
        }
        return str_dur;
    }.bind(this);
}

//TESTED
function viewCycleHistory(){
    // the window itself
    this.modal_historico_rega = document.getElementById("modal_historico_rega");
    this.ul_historico_rega = document.getElementById("ul_historico_rega");
    
    //actions
    this.btn_close_modal_hist_ciclos = document.getElementById("btn_close_modal_hist_ciclos");

    this.btn_close_modal_hist_ciclos.onclick = function() {
        hide(this.modal_historico_rega);
    }.bind(this);

    this.model = {};

    this.get_sectors_html = function(cycle){
        let html_to_return = "";
    
        if (cycle.sectors.length > 0){
            for (let id in cycle.sectors){
                let new_sector = new WateredSector(cycle.sectors[id]);
    
                html_to_return = html_to_return + `
    <li>
        <div class="hist_setor_lin_data">
            <span class="col-setor-inicio">${new_sector.start()}</span>
            <span class="col-setor-nome">${new_sector.data.name}</span>
            <span class="col-setor-duracao">${new_sector.duracao()}</span>
            <span class="col-setor-fim">${new_sector.end()}</span>
            <span class="col-setor-estado">${new_sector.data.status}</span>
        </div>
    </li>`;
                new_sector = null;
            }
        }else{
            html_to_return = "<li>sem setores</li>";
        }
    
        return html_to_return;
    }.bind(this);

    this.addCycleUI = function(cycle, fragment){
        let self = this;
        let newChild = document.createElement("li");

        addClass(newChild.classList, "historico_rega_linha");
        let suffix = `li_hist_ciclo_${cycle.data.cycleid}_${cycle.data.current_run}`;
        newChild.id = `${suffix}`;

        newChild.innerHTML = `
    <div class="hist-ciclo-data" ">
        <div class="historico_rega_linha_ciclo">
            <span class="col-inicio">${cycle.start()}</span>
            <span class="col-nome">${cycle.data.name}</span>
            <span class="col-estado">${cycle.data.status}</span>
            <span class="col-fim">${cycle.end()}</span>
        </div>
        <div id="div-hist-setores-ciclo-${cycle.data.cycleid}_${cycle.data.current_run}" class="hist_setores_ciclo hide" >
            <div class="hist_setor">
                <span class="col-setor-inicio">Inicio</span>
                <span class="col-setor-nome">Setor</span>
                <span class="col-setor-duracao">Duração</span>
                <span class="col-setor-fim">Fim</span>
                <span class="col-setor-estado">Estado</span>
            </div>
            <div class="hist_setor_data"">
                <ul>
                    ${self.get_sectors_html(cycle.data)}
                </ul>
            </div>
        </div>
    </div>`;

        fragment.appendChild(newChild);

        self.cycleElements[suffix] = {};
        self.cycleElements[suffix].element = newChild;

        self = null;
        return suffix;
    }.bind(this);

    this.view_cycle_details_onclick = function(event){
        let id = getID(event.currentTarget.id, "li_hist_ciclo_");
        let element = document.getElementById("div-hist-setores-ciclo-" + id);
        if (isVisible(element)){
            hide(element);
        } else {
            show(element);
        }
        element = null;
        id = null;
    }.bind(this);

    this.addCycleUIElements = function(suffix){
        let element = document.getElementById(suffix);
        if (element) {
            element.onclick = this.view_cycle_details_onclick;
            element = null;
        }
        else{
            log.error("sufixo desconhecido: " + suffix);
        }
    }.bind(this);

    this.get_data = function(response){
        let self = this;
        self.model["cycles"] = [];
        if (response.status === 200){
            let char4 = response.response.substring(0,4);
            if (char4 !== "time" && char4 !== "Erro"){
                self.model = JSON.parse(response.response);
            }
        }
        this.render(self.model);
        show(self.modal_historico_rega);
    }.bind(this);

    this.show = function(){
        let self = this;
        REST_API.call_server_sync(self.url, self.get_data)
    }.bind(this);


    this.render = function(model){
        let self = this;
        // clean screen
        self.ul_historico_rega.innerHTML = "";
        let fragment = document.createDocumentFragment();

        let length = model.cycles.length;
        let haveData = length > 0;
        let newChild = null;

        if (!haveData){
            newChild = document.createElement("li");
            newChild.style.width = "90vw";
            newChild.style.justifyContent="center";
            newChild.innerHTML = "Não há ciclos registados nos ultimos 15 dias.";

            fragment.appendChild(newChild); 

        }else{
            self.cycleElements = {};
            for (let id in model.cycles){
                self.addCycleUI(new WateredCycle(model.cycles[id]), fragment);
            }
        }

        self.ul_historico_rega.appendChild(fragment);

        for(let suffix in self.cycleElements){
            self.addCycleUIElements(suffix);     
        }
        fragment = null;
        newChild = null;
    }.bind(this);

    this.elements = [];
    this.url = REST_API.build_url(APP_CONTROLLER, CMD_HISTORY);
}
