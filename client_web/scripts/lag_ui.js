"use strict";

function removeClass(classList, class_){
    if (classList.contains(class_)){
        classList.remove(class_);
    }
}
function addClass(classList, class_){
    if (!classList.contains(class_)){
        classList.add(class_);
    }
}

function isVisible(element){
    return !element.classList.contains("hide")
}

function hide(object){
    addClass(object.classList, "hide");
}

function show(object){
    removeClass(object.classList, "hide");
}

function disable(element){
    element.setAttribute("disabled", "")
}

function enable(element){
    element.removeAttribute("disabled")
}

let slideUp = (elem, class_name) => {
    elem.style.maxHeight = '0';
    removeClass(elem.classList, class_name);
    elem.style.opacity = '0';
};

let slideDown = (elem, class_name) => {
    elem.style.maxHeight = null;
    addClass(elem.classList, class_name);
    elem.style.opacity   = '1';
};

function setMaterialOption(elementList, value){
    let o, ro, ri = -1;
    for(let i = 0, len = elementList.length;i < len; i = i +1){
        o = elementList[i];
        if(o.value === value){
            ro = o;
            ri = i;
            o.parentNode.MaterialRadio.check();
        }else{
            o.parentNode.MaterialRadio.uncheck();
        }
    }
    o = null;
    return {value : ro, index : ri}
}

function setStandardOption(elementList, value){
    let o, ro, ri = -1;
    for(let i = 0, len = elementList.length;i < len; i = i +1){
        o = elementList[i];
        if(o.value === value){
            ro = o;
            ri = i;
            o.checked = true;
            o.setAttribute("checked","")
        }else{
            o.checked = false;
            o.removeAttribute("checked")
        }
    }
    o = null;
    return {value : ro, index : ri}
}

function readOption(elementList){
    let o, v, inx = -1;
    for(let i = 0, len = elementList.length;i < len; i = i + 1){
        o = elementList[i];
        if(o.checked){
            inx = i;
            v = o.value;
            break;
        }
    }
    o = null;
    return {value: v, index: inx};
}

function setElementValue(element, value){
    switch (element.nodeName ){
        case "INPUT":
            element.value = value;
            break
        default:
            element.innerHTML = value;
            break;
    }
}

function unixTimeToHTMLDateFormatter (ts){
    return moment.unix(ts).local().format(ISO8601_DATE_FORMAT);
}

function validateDateField(element, errorElement){
    let val = element.value;
    let aDate   = moment(val, ISO8601_DATE_FORMAT, true); //'yyyy-mm-ddThh:mm'
    let isValid = aDate.isValid();
    let errorMsg = "";

    if (isValid){
        errorElement.innerHTML = "";
    }else{
        switch(aDate.invalidAt()){
            case -1: //no overflow
                errorMsg = "data com erro";
                break;
            case 0: //year
                errorMsg = "data com ano incorreto";
                break;
            case 1: //month
                errorMsg = "data com mês incorreto";
                break;
            case 2: //day
                errorMsg = "data com dia incorreto";
                break;
            case 3: //hour
                errorMsg = "data com hora incorreta";
                break;
            case 4: //minute
                errorMsg = "data com minutos incorretos";
                break;
        }
        //mostrar msg
        errorElement.innerHTML = errorMsg;
        show(errorElement);
    }
    return { date: aDate, valid: isValid, value: val};
}

function clearMaterialOptionList(optionsList){
    for(let i=0, len = optionsList.length; i < len; i++){
        optionsList[i].parentNode.MaterialRadio.uncheck();
    }
}

function clearMaterialCheckboxList(optionsList){
    for(let i=0, len = optionsList.length; i < len; i++){
        optionsList[i].parentNode.MaterialCheckbox.uncheck();
    }
}

function setMaterialOptionList(optionsList, optionsArr){
//assume que o optionsArr tem o mesmo numero de elementos que a optionsList
//se não tiver dá asneira
    for(let i=0, len = optionsList.length; i < len; i++){
        if (optionsList[i].value === optionsArr[i]){
            optionsList[i].parentNode.MaterialRadio.check();
            break;
        }
    }
}

function setMaterialCheckboxList(optionsList, optionsArr){
//assume que o optionsArr tem o mesmo numero de elementos que a optionsList
//se não tiver dá asneira
    for(let i=0, len = optionsList.length; i < len; i++){
        if (optionsArr.includes(optionsList[i].value)){
            optionsList[i].parentNode.MaterialCheckbox.check();
        }
    }
}

function getOptionsArray(optionsList){
    let spec_days = "";
    for(let i=0, len = optionsList.length; i < len; i++){
        if (optionsList[i].checked){
            spec_days += (i > 0)? "|":"";
            spec_days += optionsList[i].value;
        }
    }
    return spec_days;
}

function checkMaterialCheckbox(element){
    element.parentNode.MaterialCheckbox.check()
}
function uncheckMaterialCheckbox(element){
    element.parentNode.MaterialCheckbox.uncheck()
}

function checkMaterialSwitch(element){
    element.parentNode.MaterialSwitch.on()
}
function uncheckMaterialSwitch(element){
    element.parentNode.MaterialSwitch.off()
}
function disableMaterialSwitch(element){
    element.parentNode.MaterialSwitch.disable()
}
function enableMaterialSwitch(element){
    element.parentNode.MaterialSwitch.enable()
}

function setVisibility(element, isVisible){
    if (isVisible) {
        show(element);
    }else{
        hide(element);
    }
}

function addListener(event_type, arr, fun){
    let o;
    for (let i=0, len=arr.length; i < len; i=i+1){
        o = arr[i];
        o.addEventListener(event_type, fun);
    }
    o = null;
}
function clearListener(event_type, arr){
    addListener(event_type, arr, null);
}

function nl2br (str, is_xhtml) {
     //let breakTag = (is_xhtml || typeof is_xhtml === 'undefined') ? '<br />' : '<br>';
     //let res = (str + '').replace(/([^>\r\n]?)(\r\n|\n\r|\r|\n)/g, '$1' + breakTag + '$2');
     let res = str.replace(/([^>\r\n]?)(\r\n|\n\r|\r|\\n)/g, "");
     return res;
} 
function stripNewLineString (str) {
     let res = (str + "").replace(/([^>\r\n]?)(\r\n|\n\r|\r|\\n|\n)/g, "");
     return res;
}

const replaceNewLineSymbol = (str) => (str + "").replace("\n", "");

