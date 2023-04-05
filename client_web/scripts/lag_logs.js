"use strict";

//TESTED
var logs = function() {

    // SETUP DO OBJETO
    this._evt_to_attr = {};
    this._evt_to_attr[LOG_ENTRY_TYPE.ALERT] = "alert-client";
    this._evt_to_attr[LOG_ENTRY_TYPE.LOG] = "log-client";
    this._evt_to_attr[LOG_ENTRY_TYPE.ERROR] = "error-client";
    this._evt_to_attr[LOG_ENTRY_TYPE.SERVER] = "log-server";

    this.screen = {};
    this.screen["alert-client"] = { no_data_msg: "Não existem alertas.",
                                    data: new CacheObject({key: BD_CLIENT_ALERT, maxSize: 100})};
    this.screen["log-client"]   = { no_data_msg: "Não existem logs.",
                                    data: new CacheObject({key: BD_CLIENT_LOG, maxSize: 100})};
    this.screen["error-client"] = { no_data_msg: "Não existem erros.",
                                    data: new CacheObject({key: BD_CLIENT_ERROR, maxSize: 100})};
    this.screen["log-server"]   = { no_data_msg: "Não existe log significativo no servidor (Erros e Warnings).",
                                    data: new CacheObject({key: BD_SERVER_LOG, maxSize: 100}) };

    //TESTED
    this._opened_tab_name = undefined;
    this._opened_tab = undefined;
    this._open_accordion = function(evt){
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_open"));
        this._open_tab(this.screen[tgt]);
    }.bind(this);

    //TESTED
    this._open_tab = function(tab_name){
        let self = this;
        let s = self.screen[tab_name];
        if (self._opened_tab) self._closeTab(self._opened_tab_name);
        slideDown(s.panel, "alertas-coisas_open");         
        show(s.close_btn);
        hide(s.open_btn);
        self.showAction(s.data, s.del_btn); 
        self._opened_tab_name = s.panel.id;
        self._opened_tab = s.panel;
        s._last_y_pos = 0.0;
        s._last_elem = s.data.tail;
        s = null;
    }.bind(this);

    //TESTED
    this._close_accordion = function(evt){
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_close"));
        this._closeTab(tgt);
    }.bind(this);

    //TESTED
    this._toggle_accordion = function(evt){
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_header"));
        let s = this.screen[tgt];
        if(s.panel.clientHeight === 0){
            this.render(tgt);
            this._open_tab(tgt);
        }else{ 
            this._closeTab(tgt);
        }
        s = null;
    }.bind(this);

    //TESTED
    this._closeTab = function(tab_name){
        let s = this.screen[tab_name];
        slideUp(s.panel, "alertas-coisas_open");           
        hide(s.close_btn);
        hide(s.del_btn);
        show(s.open_btn);
        this._opened_tab_name = undefined;
        this._opened_tab = undefined;      
        s._last_y_pos = undefined;
        s._last_elem = undefined;
        s = null;
    }.bind(this);

    //TESTED
    this._del = function(evt){
        evt.cancelBubble = true;
        let id = evt.currentTarget.id;
        let tgt = id.slice(0, id.indexOf("_del"));
        let s = this.screen[tgt];
        if (window.confirm("Queres apagar estes registos?")){
            s.data.clear();
            this.render(tgt);
        }
        s = null;
    }.bind(this);

    //REVIEW - eventualmente temos que ver o resize também
    //TESTED
    this.onscroll = function(evt){
        /////////////
        let s = this.screen[this._opened_tab_name];
        let el = s._last_elem;
        let content_scrolled = s.panel.scrollTop;
        if (!el || el._yTop === undefined){
            el = s.data.tail;
            s._last_y_pos = 0.0;
        }

        if (content_scrolled > s._last_y_pos){  //PAGEUP
            while (el._yTop < content_scrolled){
                if (el.previous) el = el.previous; //é ver a linked list de cached data, mas estamos a mostar do fim para o principio 
                else break;
            }
        }else{  //PAGEDOWN
            while (el._yTop > content_scrolled){
                if (el.next) el = el.next; //é ver a linked list de cached data, mas estamos a mostar do fim para o principio 
                else break;
            }
        }
        s._last_y_pos = content_scrolled;
        s._last_elem = el;

        // noinspection JSUnresolvedVariable
        s.accordion.firstElementChild.dataset.badge = `${(!el.lineNum)?"1":el.lineNum}/${s.data.length}`;
        s = null;el = null;

    }.bind(this);

    //SETUP EVENTOS
    let s = null;
    for(let key in this.screen){
        s = this.screen[key];
        s.panel = document.getElementById(key);
        s.accordion = document.getElementById(`${key}_header`),
        s.list = document.getElementById(`${key}_list`);
        s.del_btn = document.getElementById(`${key}_del`);
        s.open_btn = document.getElementById(`${key}_open`);
        s.close_btn = document.getElementById(`${key}_close`);
        s.del_btn.onclick = this._del;
        s.open_btn.onclick = this._open_accordion;
        s.close_btn.onclick = this._close_accordion;
        s.accordion.onclick = this._toggle_accordion;
        s.panel.onscroll = this.onscroll;
        s._last_y_pos = undefined;
        s._last_elem = undefined;
        s._last_uuid = undefined;
    }

    // RENDER DO OBJETO PARA O ECRAN
    //TESTED
    this.showAction = function(data, element){
        if(data.length > 0){
            show(element);
        }else{
            hide(element);
        }
    };

    //TESTED
    this._aux_render = function(tab_name){
        let fragment = document.createDocumentFragment();
        let s = this.screen[tab_name];
        let cachedData = s.data;
        let length = cachedData.length;
        let haveData = length > 0;
        let list_ui = s.list;
        let newChild = null;

        if (!haveData){
            newChild = document.createElement("li");
            newChild.textContent = s.no_data_msg;
            fragment.appendChild(newChild);   
            list_ui.innerHTML = "";
            list_ui.appendChild(fragment);
            delete s.accordion.firstElementChild.dataset.badge;
        }else{
            let firstLine = 1;
            let iter = length - list_ui.children.length;
            let i = 0;
            if (iter < 0){  //apagamos elementos da lista, se houver a mais.  podiamos esconder mas era uma lógica mais complicada aa algo que vai crescer qb...
                i = Math.abs(iter);
                while(i > 0){
                    list_ui.removeChild(list_ui.children[0]);
                    i -= 1;
                }
            }else{
                while(iter > 0){  //listamos do mais recente para o mais antigo, mas só acrescentamos as linhas que faltam para o total
                    newChild = document.createElement("li");
                    newChild.classList.add("linearize");
                    fragment.appendChild(newChild);
                    iter -= 1;
                }
            }
            list_ui.appendChild(fragment);
            if (!s._last_y_pos) s._last_y_pos = 0;

            let foundFirst = false;
            let pos = 0;

            //percorremos a lista no máximo 200 vezes
            let d = cachedData.tail;
            s._last_elem = d;  //posicionamos no primeiro elemento
            iter = 0;
            while(d){ // e percorremos a lista
                d.li_elem = list_ui.children[iter];
                d.li_elem.textContent = d.data;
                d.lineNum = iter + 1;
                d._getBoundingClientRect = d.li_elem.getBoundingClientRect();

                d._yTop = pos;
                pos += d._getBoundingClientRect.height;

                if (!foundFirst && pos > s._last_y_pos ){
                    s._last_elem = d;
                    foundFirst = true;  
                }

                d = d.previous;
                iter += 1;
            }
            d = null;
            firstLine = s._last_elem.lineNum;
            s.accordion.firstElementChild.dataset.badge = `${firstLine}/${length}`;
        }
        fragment = null;
        s = null;
        cachedData = null;
        list_ui = null;
        newChild = null;
    };

    //TESTED
    this._handleButtons = function(key){
        let s = this.screen[key];
        if (!isVisible(s.panel) || s.panel.clientHeight === 0 ){ 
            hide(s.del_btn);
        }else{
            this.showAction(s.data, s.del_btn);                
        }
        s = null;
    };

    // TESTED
    this.render = function (attr, all = false){
        let self = this;
        if (all){
            for(let key in self.screen){
                self._aux_render(key);
                self._handleButtons(key);
            }
        }else{
            self._aux_render(attr);
            self._handleButtons(attr);
        }
    };

    //TESTED
    this.populate = function(){
        let self = this; 
        for(let key in self.screen){
            self.screen[key].data.populate();
        }
    };

    //TESTED
    this.newEntry = function(msg, key){
        if (!key){
            // noinspection JSUnresolvedFunction
            key = new UUID(1).format();
        }
        return {key: key, message: msg};
    };

    //TESTED
    this.addEntry = function(entryType, msg){
        let m = "";
        if (!(msg instanceof String)){
            let s1 = m = moment().format(DATE_LOG_FORMAT);
            if (msg.message) m = `${s1}:${JSON.stringify(msg.message)}`;
            else m = s1;
        }else{ 
            m = `${moment().format(DATE_LOG_FORMAT)}:${msg}`; 
        }
        let attr = this._evt_to_attr[entryType];
        this.screen[attr].data.addObj(msg.key, m);
        console.log(m);  //REVIEW - isto é a retirar para produção 
        this.render(attr);
    };

    //TESTED
    this.error = function(error){
        let stack = "";
        let message = "";
        if (error instanceof ErrorEvent){
            if (error && error.error && error.error.stack) stack = error.error.stack;
            message = error.error.toString();
            if(stack){ message += stack; }
        }else{
            stack = error.stack;
            message = error.toString();
            
            if(stack){ message += stack; }
        }
        this.addEntry(LOG_ENTRY_TYPE.ERROR, this.newEntry(message));
    }.bind(this);

    //NOTTESTED
    this.STCLogArrived = function(message){
        let payload = message.body.message;
        this.addEntry(LOG_ENTRY_TYPE.SERVER, payload);
        MAIN.updateDate(moment().format())
    }.bind(this)
};

