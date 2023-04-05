"use strict";

var rest_api = function(){

    this.call_sync = function(url, callback){
        const response = new XMLHttpRequest();
        response.open("POST", url, false);
        response.setRequestHeader('Content-Type', 'application/json');
        response.send(url);
        callback(response);
        MAIN_VIEW.hide_loader();
    }.bind(this);

    // message   - json string 
    // {op_id: "get_id"}
    // callback  - one parameter function, returning a string with server response
    this.call_server = function(url, callback){
        const response = new XMLHttpRequest();
        response.open("POST", url);
        response.responseType = 'text';
        response.onload = callback;
        response.setRequestHeader('Content-Type', 'application/json');
        
        response.send(url);

    }.bind(this);

    // message   - json string 
    // {op_id: "get_id"}
    // callback  - one function that receives the server response object as a parameter
    this.call_server_sync = function(url, callback){
        MAIN_VIEW.show_loader();
        setTimeout(this.call_sync, 0, url, callback )
    }.bind(this);

    // identify current environment (tst ou prd)
    // assumes TEST is already intialized
    this.build_url = function(application, command){
        let environment = "";
        if (TEST){
            environment = "tst/"
        }else{
            environment = "prd/"
        }
        let url = PROTOCOL + HOST;
        if (!command){
            url = url + "/" + environment + application;
        }else{
            url = url + "/" + environment + application + "/" + command;
        }
        return url;
    }.bind(this);

    try{       

    }catch(err){
        log.error(err);
    }
};
