"use strict";

var main_view = function(){

	this.footer_default_yes_response_snackbar = document.getElementById("footer-default-yes-response-snackbar");
	this._menu_coisas = document.getElementById("open-menu");
	this.menu = document.getElementById("dropDownMenu");
	this.swipe_container = document.getElementById("page-container");
	this._nav_helper_index_to_name = {
		0: "tempo-panel",
		1: "rega-panel", 
		2: "cenas-panel",
		3: "coisas-panel"
	};
	this._nav_helper_name_to_index = {
		"tempo-panel" :0,
		"rega-panel"  :1, 
		"cenas-panel" :2,
		"coisas-panel":3
	};
	this._nav = {
		"rega-panel"	: {item: document.getElementById("navitem_rega-panel"), panel: document.getElementById("rega-panel"), originalClass:["mdl-tabs__panel", "page"]},
		"tempo-panel"	: {item: document.getElementById("navitem_tempo-panel"), panel: document.getElementById("tempo-panel"), originalClass:["mdl-tabs__panel", "page"]},
		"cenas-panel"	: {item: document.getElementById("navitem_cenas-panel"), panel: document.getElementById("cenas-panel"), originalClass:["mdl-tabs__panel", "page"]},
		"coisas-panel"	: {item: document.getElementById("navitem_coisas-panel"), panel: document.getElementById("coisas-panel"), originalClass:["mdl-tabs__panel", "coisas-panel"]}
	};
	this._menu = {
		"coisas-geral"		: {menu: document.getElementById("menuitem-coisas-geral"), panel: document.getElementById("coisas-geral")},
		"coisas-alertas"	: {menu: document.getElementById("menuitem-coisas-alertas"), panel: document.getElementById("coisas-alertas")},
		"coisas-about"		: {menu: document.getElementById("menuitem-coisas-about"), panel: document.getElementById("coisas-about")},
		"coisas-controlador": {menu: document.getElementById("menuitem-coisas-controlador"), panel: document.getElementById("coisas-controlador")}
	};

	this.StatusMsg = new (function StatusMsg(){
	    this.status_msg = document.getElementById("status_msg");

	    this.set = function(msg){
	        this.status_msg.innerHTML=msg;
	    };

	    this.setTemp = function(msg, timeOut){
	        this.set(msg);
	        var _clear = setTimeout(this.clear, timeOut * 1000);
	    };

	    this.clear = function(){ status_msg.innerHTML=""; };
	})();

	this._tab_panel = undefined;
	this._menu_panel = undefined;
	this._tab_name = window.localStorage.getItem('ACTIVE_TAB');
	this._menu_name = window.localStorage.getItem('ACTIVE_MENU');
	this._menu_beeing_open = false;
	this._tab_index = 0;
	this._max_tab_index = 3;
	this._prev_tab_panel = undefined;

	this.__reset_page = function(panel){
		let cls = [...panel.classList];
		let self = this;
		panel.classList.remove(...cls);
		panel.classList.add(...self._nav[panel.id].originalClass);
		if(self._prev_tab_panel){ // o raio do evento está a ser chamado duas vezes....e ainda não percebi porquê
			self._prev_tab_panel.panel.classList.add("hide");
			self._prev_tab_panel.panel.classList.remove("is-active");
		}
		panel.classList.add("is-active");		
	};

	this._animation_end = function(evt){
		let el = evt.currentTarget;
		this.__reset_page(el);
	}.bind(this);

	this._deactivate_tab = function(tab_obj, outClass){
    	removeClass(tab_obj.item.classList, "is-active");
    	this.__reset_page(tab_obj.panel);
    }.bind(this);

	this._nav_to = function(event){
		let elem = event.target.parentElement;
		let panel_name = elem.id.slice(8);
		this.__nav_to(this._nav[panel_name].panel);
		elem = null;
	}.bind(this);

	this.__nav_to = function(nav_to_elem, inClass, outClass){
		let self = this;
		if(self._tab_panel) {
			self._prev_tab_panel = self._nav[self._tab_name];
			self._deactivate_tab(self._prev_tab_panel, outClass);
		}
	    show(nav_to_elem);
	    if (inClass) {	
			void nav_to_elem.offsetWidth; //isto é para trigger e repetição da animação.
	    	nav_to_elem.classList.add(inClass, "is-active"); 
		}else{
	 	  	nav_to_elem.classList.add("is-active");
	    }
	    addClass(self._nav[nav_to_elem.id].item.classList, "is-active");
	    window.localStorage.setItem('ACTIVE_TAB', nav_to_elem.id);
	    self._tab_panel = nav_to_elem;
	    self._tab_name = nav_to_elem.id;
	    self._tab_index = self._nav_helper_name_to_index[self._tab_name];
	}.bind(this);

	this._nav_to_menu = function(event){
		let elem = event.target;
		let panel_name = elem.id.slice(9);
		let self = this;

		if(self._menu_panel) hide(self._menu_panel); 

		if (self._tab_name){
			if (self._tab_name !== "coisas-panel") self._deactivate_tab(self._nav[self._tab_name]);
			self.__nav_to(self._nav["coisas-panel"].panel);
		}
		self.__nav_to_menu(self._menu[panel_name].panel);
		elem = null;
	}.bind(this);

	this.__nav_to_menu = function(nav_to){
	    show(nav_to);
	    window.localStorage.setItem('ACTIVE_MENU', nav_to.id);
	    this._menu_panel = nav_to;
	    this._menu_name = nav_to.id;
	    this._menu_beeing_open = false;
	}.bind(this);

	this._ini_tab_nav = function(){
		let self = this;
		let obj = null;
	    for(let key in self._nav){
	    	obj = self._nav[key];
	    	self._deactivate_tab(obj);
	    	obj.panel.classList.remove("is-active");
	    	hide(obj.panel);
	    	obj.item.onclick = self._nav_to;
	    	obj.panel.addEventListener('webkitAnimationEnd', self._animation_end);
	    }		
	    self._menu_coisas.onclick = this._open_menu;
	};

	this._ini_menu_nav = function(){
		let obj;
		let self = this;
	    for(let key in self._menu){
	    	obj = self._menu[key];
	    	removeClass(obj.panel.classList, "is-active");
	    	hide(obj.panel);
	    	obj.menu.onclick = this._nav_to_menu;
	    }
	    obj = null;	
	};

	this.closeMenu = function (){
	    removeClass(this.menu.classList,"open");
	};

	this._open_menu = function (evt){
		this._menu_beeing_open = true;
	    this.menu.classList.toggle("open");
	    setTimeout(() => this._menu_beeing_open = false, 350);
	}.bind(this);

	// Close the dropdown if the user clicks outside of it
	window.onclick = function(e) {
	    if (e.target.parentElement.id !== "navitem_coisas-panel" && !this._menu_beeing_open) {
	        this.closeMenu();
	    }
	}.bind(this);

	this.ini_application_ui = function(){
		let self = this;
	    self._ini_tab_nav();
	    self._ini_menu_nav();
 
	 	if (self._tab_name){
			self.__nav_to(self._nav[self._tab_name].panel);
		}
		if(this._menu_name){
			self.__nav_to_menu(self._menu[self._menu_name].panel);
		}
	};

	this.swipedetect = function(el, callback){
	    let touchsurface = el,
	    swipedir,
	    startX,
	    startY,
	    threshold = 150, //required min distance traveled to be considered swipe
	    restraint = 50, // maximum distance allowed at the same time in perpendicular direction
	    allowedTime = 300, // maximum time allowed to travel that distance
	    startTime,
	    handleswipe = callback || function(swipedir){};
	  
	    touchsurface.addEventListener('touchstart', function(e){
	        let touchobj = e.changedTouches[0];
	        swipedir = 'none';

	        startX = touchobj.pageX;
	        startY = touchobj.pageY;
	        startTime = new Date().getTime(); // record time when finger first makes contact with surface
	    }, {passive: true});
	    
	    touchsurface.addEventListener('touchend', function(e){
	        let touchobj = e.changedTouches[0],
	        distX = touchobj.pageX - startX, // get horizontal dist traveled by finger while in contact with surface
	        distY = touchobj.pageY - startY, // get vertical dist traveled by finger while in contact with surface
	        elapsedTime = new Date().getTime() - startTime; // get time elapsed
	        if (elapsedTime <= allowedTime){ // first condition for swipe met
	            if (Math.abs(distX) >= threshold && Math.abs(distY) <= restraint){ // 2nd condition for horizontal swipe met
	                swipedir = (distX < 0)? 'left' : 'right'; // if dist traveled is negative, it indicates left swipe
	            }
	        }
	        handleswipe(swipedir);
	        if(swipedir !== "none") e.preventDefault();
	    }, false);
	}.bind(this);
	  
	this.swipe = function(swipedir){
	    //swipedir contains either "none", "left", "right", "top", or "down"
	    let newindex = 0;
	    let move = true;
	    let inClass = undefined, outClass = undefined;
	    let self = this;
	    switch(swipedir){
	    	case "left":
	    		if (this._tab_index > 0){
	    			newindex = self._tab_index - 1;
	    		}else{
	    			newindex = self._max_tab_index;
	    		}
				inClass = "page-moveFromRight";
				outClass = "page-fade";
	    		break;
	    	case "right":
	    		if (this._tab_index < self._max_tab_index){
	    			newindex = self._tab_index + 1;
	    		}else{
	    			newindex = 0;
	    		}
	    		inClass = "page-moveFromLeft";
	    		outClass = "page-fade";
	    		break;
	    	case "top":
	    	case "down":
	    	case "none":
	    		move = false;
	    		break;
	    }
		if (move) self.__nav_to(self._nav[self._nav_helper_index_to_name[newindex]].panel, inClass, outClass);
	}.bind(this);

	this.swipedetect(this.swipe_container, this.swipe);
};

