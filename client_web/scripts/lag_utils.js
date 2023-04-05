"use strict";

function isInt(value) {
  if (isNaN(value)) {
    return false;
  }
  let x = parseFloat(value);
  return (x | 0) === x;
}

function config_client_id(base){
    if (TEST){
        CLIENT_ID = base + "_tst";
    }else{
        CLIENT_ID = base + "_prd";
    }
    return CLIENT_ID
}

function is_test_env(){
    let aux_url = window.location.href;
    let paths = aux_url.split('/');
    let elem = "";
    let found = false;
    for (var i = 0, n = paths.length; i < n; ++i){
        elem = paths[i];
        if (elem === "tst"){
            found = true;
            break;
        }
    }
    return found;
}

const js_date_to_unix = function(date){
    return Math.floor(date.getTime() / 1000);
};
const js_date_to_unix_adjusted = function(date){
    return Math.floor((date.getTime() + DELTA_TIME_MS) / 1000);
};
const unix_to_js_date = function(timestamp){
    return new Date(timestamp * 1000);
}
const unix_to_js_date_adjusted = function(timestamp){
    return new Date((timestamp * 1000) + DELTA_TIME_MS);
}

const get_unix_now_time = function(){
    return js_date_to_unix(new Date());
}
const get_unix_now_time_adjusted = function(){
    return js_date_to_unix_adjusted(new Date());
}

const get_js_now_time_adjusted = function(){
    return unix_to_js_date(js_date_to_unix_adjusted(new Date()));
}

const get_minutes_from_unix = function(timestamp){
    return timestamp / 60;
}

const date_to_iso8601 = function (date){
    let d = date.toISOString().split('.'); 
    return d[0] + "Z";
}

// function pad(n){return n<10 ? '0'+n : n}
const pad = function(num){
    var s = "0" + num;
    return s.substring(s.length - 2);
}

const get_time_str_from_unix = function(timestamp){
    let date = unix_to_js_date(timestamp);
    let hour = date.getUTCHours();
    let min = date.getUTCMinutes();
    return pad(hour) + ":" + pad(min);
}

// 2022-10-16T00:00
function RFC3339DateString(d){
    return d.getUTCFullYear()+'-'
         + pad(d.getUTCMonth()+1)+'-'
         + pad(d.getUTCDate())+'T'
         + pad(d.getUTCHours())+':'
         + pad(d.getUTCMinutes()) //+':'
        //  + pad(d.getUTCSeconds())+'Z'
}

function buildAndSendMessage(msgTopic, msg){
    try{
        let message = MSG.BuildMessage(msgTopic, msg);
        MQTT.SendMessage(message);
    }
    catch(err){
        log.error(err);
    }
}

const MONTH_DAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTH_NAME = ["Jan", "Fev", "Mar", "Abr", "Mai", "Jun", "Jul", "Ago", "Set", "Out", "Nov", "Dez"];

const is_leap = function(year) {
    // if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
    //     1
    // } else {
    //     0
    // }
    return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// "%Y-%m-%dT%H:%M:%S%";
 function validate_str_iso_rfc3339(str_date) {
    let v = str_date.split(/[-T:\s]/);

     if (v.length > 5) {
         return [false, -1];
     };
     let year = parseInt(v[0]);
     if (year === "NaN") {
             return [false, 0]; 
     }
     let month = parseInt(v[1]);
     if (month !== "NaN") {
        if (month < 1 || month > 12){
             return [false, 1]; 
         }
     }else{
        return [false, 1]; 
     }
     let day = parseInt(v[2]);
     if (day !== "NaN"){
         let days = MONTH_DAYS[month - 1];
         if (month === 2) {
             days += is_leap(year);
         }
         if (day < 1 || day > days){
            return [false, 2]; 
         }
     }else{
        return [false, 2]; 
     }
     let hour = parseInt(v[3]);
     if (hour !== "NaN") {
        if (hour < 0 || hour > 23){
            return [false, 3]; 
         }
     }else{
        return [false, 3]; 
     }
     let min = parseInt(v[4]);
     if (min !== "NaN"){
         if (min <0 || min > 69) {
            return [false, 4]; 
         }
     }
     else{
        return [false, 4]; 
     }
     let sec = parseInt(v[5]);
     if (sec !== "NaN") {
        if (sec < 0 || sec > 59){
            return [false, 5]; 
         }
     }else{
        return [false, 5]; 
     }
     return [true, 0];
 };

// TESTED
const addMethods = function(object, methods){
    for(let name in methods) {
        if (typeof methods[name] === "object" && !(methods[name] instanceof Element)){
            object[name] = methods[name];
            addMethods(object[name], methods[name]);
        }
        else{
            object[name] = methods[name];
        }
    }
};

function round(num, precision) {
    const f = 10 ** precision;
    return Math.round((num + Number.EPSILON) * f) / f;
}

const percentToDegree = p => p * 360;
const degreeToRadian = d => d * Math.PI / 180;
const percentToRadian = p => degreeToRadian(percentToDegree(p));

//TESTED
function CacheObject(options){
    this.data = {};
    this.head = undefined;
    this.tail = undefined;
    this.maxSize = options.maxSize;
    this.table = options.key;
    this._changed = false;
    this.length = 0;

    this.populate = function(){
        let objs_str = localStorage.getItem(this.table);
        let dt = [];
        let e = null;
        let temp = "";
        let self = this;
        if (objs_str){  
            dt = JSON.parse(objs_str); 
        }
        for (let i = 0, l = dt.length; i < l; i += 1){
            e = dt[i];
            temp = replaceNewLineSymbol(e.data);
            self.addObj(e.key, temp);
        }
        e = null;
        dt = null;
    }.bind(this);
                          
    this.save = function(){
        let start = this.head;
        let dt = [];
        while(start){
            dt.push({ key: start.key, data: start.data }) ;
            start = start.next;
        }
        localStorage.setItem(this.table, JSON.stringify(dt)); 
        this._changed = false;
        dt = null;
        start = null;
    }.bind(this);

    this.have = function(key) { return key in this.data; };

    this.addObj = function(key, data) {
        let self = this;
        if (key in self.data){ return; } //guard - if ey exists, do nothing
        //REVIEW DEBUG - para retirar
        if(!key){
            console.log("olha o null!")
        }
        self.length += 1;
        if (self.length > self.maxSize){  // delete older object
            self.nextObj();
        }
        let newOBJ= { key: key, data: data };
        self.data[key] = newOBJ;

        // Set this as the oldest request if it is the first request
        if (!self.head) {
            self.head = newOBJ;
        }
        // If this isn't the first request, add it to the end of the list
        if (self.tail) {
            self.tail.next = newOBJ;
            newOBJ.previous = self.tail;
        }

        self.tail = newOBJ;
        self._changed = true;
        newOBJ = null;
    };

    this.getObj = function(key){
        return this.data[key];
    };

    this.nextObj = function() {
        let self = this;
        // If we don't have any requests, undefined is returned
        if (self.head) {
            let obj = self.head;
            self.head = obj.next;
            delete self.data[obj.key];

            // Make sure we don't hang on to references to users
            // that are out of the queue
            if (self.head) {
               delete self.head.previous;
            }
            // This is the last request in the queue so "empty" it
            if (obj === self.tail) {
               self.tail = undefined;
            }
            self.length -= 1;
            self._changed = true;
            return obj;
        }
    };

    this.removeObj = function(key) {
        let self = this;
        if (!key){
            key = self.head.key;
        }
        if(key){
            let obj = self.data[key];
            delete self.data[key];

            if (obj.previous) {
                obj.previous.next = obj.next;
            }
            if (obj.next) {
                obj.next.previous = obj.previous;
            }
            obj = null;
        }
        self.length -= 1;
        self._changed = true;

    };

    this.clear = function(){
        let self = this;
        while(self.head){
            self.nextObj();
        }
        self.save();
    };
    setInterval(this.save, 2000);

    return this;
}

const debug_callback = function debug_callback(json){console.log(json)};

//TESTED
function ThreadSimul(name, seconds, callback_func){
    this.callback_func = callback_func;
    this.seconds = seconds;
    this.name = name;
    this.interval = undefined;

    let isRunning = false;

    // TESTED
    this._run = function (){
        // thread executes each defined seconds to process events
        try{
            if (this.callback_func) {
                this.callback_func();
            }
        }
        catch(error){
            error.message = ["Erro no call back do timeout: ", this.name, error.message].join("");
            log.error(error);
        }
    }.bind(this);

    // TESTED
    this.start = function (){
        clear_interval(this.interval);
        this.interval = setInterval(this._run, this.seconds * 1000);  //wakesup every 1 second
        isRunning = true;
    };

    // NOTTESTED
    this.reset = function (){
        clear_interval(this.interval);
        this.interval = setInterval(this._run, this.seconds * 1000);  //wakesup every 1 second
        isRunning = true;
    };
    //# TESTED
    this.stop = function (){
        clearInterval(this.interval) ;
        isRunning = false;
    };

    this.isRunning = function(){return isRunning;}
}

const clear_interval = (interval) => {
    if (typeof interval !== "undefined"){
        clearInterval(interval);
    }
}

const debounce = (func, delay) => {
    let inDebounce;
    return function(){
        const context = this;
        const args = arguments;
        clearTimeout(inDebounce);
        inDebounce = setTimeout(()=>func.apply(context, args), delay);
    }
};

const throttle = (func, limit) => {
    let lastFunc;
    let lastRan;
    return function(){
        const context = this;
        const args = arguments;
        if(!lastRan){
            func.apply(context, args);
            lastRan = Date.now();
        }else{
            clearTimeout(lastFunc);
            lastFunc = setTimeout(function(){
                if((Date.now() - lastRan) >= limit){
                    func.apply(context, args);
                    lastRan = Date.now();
                }
            }, limit - (Date.now() - lastRan))
        }
    };
};

// TESTED
function running_stat(){
    this.m_n = 0;
    this.m_oldM = 0.0;
    this.m_newM = 0.0;
    this.m_oldS = 0.0;
    this.m_newS = 0.0;
        
    this.clear = () => this.m_n = 0;
  
    this.push = function(x){
        self = this;
        self.m_n += 1;

        // See Knuth TAOCP vol 2, 3rd edition, page 232
        if (self.m_n === 1){
            self.m_newM = x;
            self.m_oldM = self.m_newM;
            self.m_oldS = 0.0;
        } else{
            self.m_newM = self.m_oldM + (x - self.m_oldM) / self.m_n;
            self.m_newS = self.m_oldS + (x - self.m_oldM) * (x - self.m_newM);
            // set up for next iteration
            self.m_oldM = self.m_newM;
            self.m_oldS = self.m_newS;
        }
    };

    this.mum_data_values = () => this.m_n;

    this.mean = () => (self.m_n > 0)? this.m_newM: 0.0;

    this.variance = () =>  (this.m_n > 1)? this.m_newS / (this.m_n - 1): 0.0; 

    this.standard_deviation = () =>  Math.sqrt( this.variance() );
}

// Iterative function to implement Binary Search
// TESTED
function binarySearch(arr, x) { 
   
    let start = 0, end = arr.length - 1; 
          
    // Iterate while start not meets end 
    while (start <= end){ 
  
        // Find the mid index 
        let mid = Math.floor((start + end) / 2); 
   
        // If element is present at mid, return True 
        if (arr[mid].x === x) return mid; 
  
        // Else look in left or right half accordingly 
        else if (arr[mid].x < x)  
             start = mid + 1; 
        else
             end = mid - 1; 
    } 
   
    return -1; 
} 

function clamp(num, min, max){
    return Math.min(Math.max(num, min), max);
}

//TESTED
function LagEvent(){
    this.observers = [];
    
    this.registerObserver = function(observer, callback){
        let i = 0, l = this.observers.length;
        let found = false;
        for(i= 0; i < l; i += 1){
            if (this.observers[i]["observer"] === observer){
                found = true;
                break;
            }
        }
        if (!found){
            this.observers.push({"observer": observer, "callback": callback});
        }
    }.bind(this);
    
    this.unregisterObserver = function(observer){
        let i = 0, l = this.observers.length;
        let index = -1;
        for(i= 0; i < l; i += 1){
            if (this.observers[i]["observer"] === observer){
                index = i;
                break;
            }
        }        
        if (index > -1){
            this.observers.splice(index, 1);
        }
    }.bind(this);

    this.notifyObservers = function(param1){
        let callback = null;
        let i = 0, l = this.observers.length;
        let ocopy = [];
        // needs to have a copy, because a race condition may existis.  its possible to call the callback from an observer that was already unregistered
        for(i= 0; i < l; i += 1){
            ocopy.push(this.observers[i]);
        }
        // notify all observers
        l = ocopy.length;
        for(i= 0; i < l; i += 1){
            try{
                callback = ocopy[i]["callback"];
                callback(param1);
            }catch(err){
                log.error(err);
            }            
        }
        ocopy = null;
        callback = null;
    }.bind(this);
}