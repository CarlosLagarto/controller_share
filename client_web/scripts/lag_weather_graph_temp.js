"use strict";

let minutesWindow = 1440;

//TESTED
let Chart = function(options){
    let _this = this;
    this._nowTS = Math.floor(get_minutes_from_unix(get_unix_now_time()));

    this.minutesWindow = options.minutesWindow;

    this._lastTS = -1;

    this.data = [];

    this.chart = d3.select(options.chartId)
        .attr('class', 'chart');

    this.interval = [_this._nowTS - this.minutesWindow, _this._nowTS];
    this.maxY = options.minY;
    this.minY = options.maxY;
    this.tickS = ["24h", "16h", "8h", "4h", "1h", ""];

    this.margin = {top: 2, right: 0, bottom: 12, left: 45};
    this.width = + this.chart.attr("width") -  this.margin.left -  this.margin.right;
    this.height = + this.chart.attr("height") -  this.margin.top -  this.margin.bottom;

    this.plot = this.chart.append("g")
        .attr("transform", `translate(${this.margin.left},${this.margin.top})`);

    this.xScale = d3.scaleLinear()
                .domain(this.interval)
                .range([0, this.width]);

    this.yScale = d3.scaleLinear()
                .domain([options.minY, options.maxY])
                .range([this.height, 0]);

    this.line = d3.line()
            .x(d => _this.xScale(d.x))
            .y(d => _this.yScale(d.y))
            .defined(d => d !== null)
            .curve(d3.curveMonotoneX);

    this.tickV = [this.interval[0],
                 this.interval[0] + Math.round(this.minutesWindow * 0.333333333),
                 this.interval[0] + Math.round(this.minutesWindow * 0.666666667),
                 this.interval[0] + Math.round(this.minutesWindow * 0.833333333),
                 this.interval[0] + Math.round(this.minutesWindow * 0.958333333),
                 this.interval[0] + this.minutesWindow];

    this.xAxis = d3.axisBottom(this.xScale)
        .tickValues(this.tickV)
        .tickFormat(t => {
                            let i = this.tickV.indexOf(t);
                            let s = "";
                            if (i !== -1) s = this.tickS[i];
                            return s;
                         })
        .tickSize([0]);

    this.axis = this.plot.append('g')
        .attr('class', 'axis')
        .attr('transform', `translate(0,${this.yScale(options.minY)})`)
        .call(this.xAxis);

    this.paths = this.plot.append('g');
    this.path = this.paths.append('path')
                .attr('class', 'line')
                .attr("d", this.line(this.data));

    this.verticalLines = this.plot.append('g');

    for (let i = 0, len = this.tickV.length - 1 ; i < len; i = i + 1){
        this.verticalLines.append('line')
            .attr('class', 'vertical-lines')
            .attr('x1', this.xScale(this.tickV[i]))
            .attr('y1', this.yScale(options.minY))
            .attr('x2', this.xScale(this.tickV[i]))
            .attr('y2', this.yScale(options.maxY));
    }

    this.circles = this.paths.append("g");

    this.fakeXAxis = this.plot.append('g');

    this.fakeXAxis.append('line')
        .attr('class', 'fake-x-axis')
        .attr('x1', 0)
        .attr('y1', this.yScale(options.xAxisYReference))
        .attr('x2', this.minutesWindow * 2)
        .attr('y2', this.yScale(options.xAxisYReference));
    this.fakeXAxis.append('text')
        .attr("class","x-axis-label")
        .attr("y", this.yScale(options.xAxisYReference) - 1)
        .attr("x", 1)
        .text(round(options.xAxisYReference, options.roundDecimals) + options.displayUnit);

    this.minmaxTemp = this.plot.append('g');

    this.updateMinMaxLines = function (){
        let self = this;
        self.minmaxTemp.remove();
        self.minmaxTemp = self.plot.append('g');
        let minY = self.minY;
        let maxY = self.maxY;
        if (!!minY){
            self.minmaxTemp.append('line')
                .attr('class', 'min-line')
                .attr('x1', 0)
                .attr('y1', self.yScale(minY))
                .attr('x2', self.minutesWindow * 2)
                .attr('y2', self.yScale(minY));
            self.minmaxTemp.append('text')
                .attr("class","min-label")
                .attr("text-anchor", "end")
                .attr("y", self.yScale(minY) + 8)
                .attr("x", -7 )
                .text(round(minY, options.roundDecimals) + options.displayUnit);
        }
        if (!!maxY){
            self.minmaxTemp.append('line')
                .attr('class', 'max-line')
                .attr('x1', 0)
                .attr('y1', self.yScale(maxY))
                .attr('x2', self.minutesWindow * 2)
                .attr('y2', self.yScale(maxY));
            let posY = self.yScale(maxY) - 2;
            if (posY - 7 < 0) posY = 7;
            self.minmaxTemp.append('text')
                .attr("class","max-label")
                .attr("text-anchor", "end")
                .attr("y", posY)
                .attr("x", - 7 )
                .text(round(maxY, options.roundDecimals) + options.displayUnit);
        }
    }.bind(this);

    this.render = function(tsNow, value) {
        let _this = this;

        let _lastTS = _this._lastTS;
        let data = _this.data;
        let duration;

        if (tsNow === undefined || value === undefined){
            // no parameters means that it's only required the refresh
            if (data.length > 0){
                tsNow = data[data.length-1].x;                
            }else{
                tsNow = _this._nowTS;
            }
        }else{
            //else, add element to array
            data.push( {x: tsNow, y: value } );
        }
        if (_lastTS === -1) _lastTS = tsNow;
        duration = tsNow - _lastTS; // time since last update
        _this._lastTS = tsNow;

        let xSlideLeft = tsNow - (_this.minutesWindow + duration);

        // Remove oldest data points
        if (data.length > 0){
            while(data[0].x < xSlideLeft){
                data.shift();
            }
        }

        let ext = d3.extent(data, d => +d.y);
        _this.maxY = ext[1];
        _this.minY = ext[0];

        // handle circles
        _this.circles.selectAll("circle").remove();

        // Shift domain
        _this.interval = [tsNow - (_this.minutesWindow + duration) , tsNow - duration];
        _this.xScale.domain(_this.interval);
        // Slide x-axis left
        _this.xAxis = d3.axisBottom(_this.xScale);

        // redraw line
        _this.line
            .x(d => _this.xScale(d.x))
            .y(d => _this.yScale(d.y));
        _this.paths.select(".line")
            .attr("d", _this.line(data));


        _this.circles.selectAll("circle").data(data)
            .enter()
            .append("circle")
            .attr('class', 'circle')
            .attr("cx", d => _this.xScale(d.x) )
            .attr("cy", d => _this.yScale(d.y) )
            .attr("r", +1.5);

        _this.updateMinMaxLines();
        ext = null;
        _this = null;
    }.bind(this);
};

//TESTED
let Wind = function(options){
    let _this = this;

    this.minutesWindow = options.minutesWindow;

    this._lastTS = -1;

    this.data = [];
    //codigo de cores tipo "heat map" em função da idade do ponto.  Mais claro á mais tempo, mais escuro á menos tempo
    //9 cores, a cada 20' de idade dá as ultimas 3 horas de que mostramos do vento
    this.colors = ["#1CFCFD","#37DCDD","#52BDBE", "#6D9D9E", "#887E7F", "#A35E60", "#BE3F40", "#D91F21", "#F50002"];
    this.chart = d3.select(options.chartId)
            .append("svg")
            .attr('class', 'wind')
            .attr("preserveAspectRatio", "xMinYMin meet")
            .attr("viewBox", "0 0 94 94")
            .classed("svg-content", true);

    this.maxY = options.minY;
    this.minY = options.maxY;

    this.margin = {top: 2, right: 2, bottom: 2, left: 2};

    this.width = + 94 -  this.margin.left -  this.margin.right;
    this.height = + 94 -  this.margin.top -  this.margin.bottom;

    let domain = [options.minWind, Math.log10(options.maxWind)];
    let radius = ( this.width <= this.height? this.width: this.height) / 2;
    this.radiusScale = d3.scaleLinear()
                            .domain(domain)
                            .range([0, radius]);

    let sTranslate = `translate(${((this.width / 2) + this.margin.left)},${((this.height / 2) + this.margin.top)})`;
    this.plot = this.chart.append("g")
        .attr("transform", sTranslate);

    this.dataLimits = [{r:Math.log10(100),class:"tempestade"},
                       {r:Math.log10(60) ,class:"forte"},
                       {r:Math.log10(30) ,class:"moderado"},
                       {r:Math.log10(10) ,class:"aragem"}];
                       
    this.circles = this.chart.append("g")
        .attr("transform", sTranslate);

    this.circles.selectAll("circle")
        .data(this.dataLimits)
        .enter().append("circle")
        .attr("class", d => d.class)
        .attr("r", d => _this.radiusScale(d.r));

    this.render = function(tsNow, wind, direction) {
        let _this = this;
        let _lastTS = _this._lastTS;
        let data = _this.data;

        if (tsNow === undefined || wind === undefined || direction === undefined){
            // se não forem passados parametros, quer dizer que é só para fazer refresh com os dados que estão no array
            // que já alguem os atualizou se quiser.
            if (data.length > 0){
                tsNow = data[data.length-1].ts;                
            }else{
                tsNow = _this._nowTS;
            }
        }else{
            //senão, adiciona o elemento ao array
            data.push( {ts: tsNow, wind: wind, dir: direction } );
        }
        
        if (_lastTS === -1) _lastTS = tsNow;
        let duration = tsNow - _lastTS;
        _this._lastTS = tsNow;

        // Remove oldest data points
        let xSlideLeft = tsNow - (_this.minutesWindow + duration);
        if (data.length > 0){
            let ts = data[0].ts;
            while(ts < xSlideLeft){
                data.shift();
                if (data.length > 0) ts = data[0].ts;
                else break;
            }
        }

        _this.plot.selectAll("circle")
            .data(data)
            .exit().remove();

        let ext = d3.extent(data, d => +d.ts);
        let maxts = ext[1];

        let fn = function(d){
            let color_index = 8 - Math.trunc((maxts - d.ts) / 20);
            color_index = (color_index >= 8)? 8: color_index;
            color_index = (color_index <= 0)? 0: color_index;
            return _this.colors[ color_index ];             
        };
        
        _this.plot.selectAll("circle")
            .data(data)
            .enter().append("circle")
            .attr("transform", d => `rotate(${(d.dir - 180)}) translate(0,${_this.radiusScale(d.wind)})`)
            .attr("r",2);

        _this.plot.selectAll("circle")
            .data(data).each(function(d, i) {
                                let circle = d3.select(this);
                                circle.style("stroke", d => fn(d)); });
        _this = null;
    }.bind(this);

};

//TESTED
let Needle = function(props) {

    this.svg = props.svg;
    this.group = this.svg.append('g');
    this.len = props.len;
    this.radius = props.radius;
    this.x = props.x;
    this.y = props.y;

    this.render = function () {
        let self = this;
        self.group.attr('transform', `translate(${self.x},${self.y})`);
        self.group
          .append('circle')
          .attr('class', 'c-chart-gauge__needle-base')
          .attr('cx', 0)
          .attr('cy', 0)
          .attr('r', self.radius);

        self.group
          .append('path')
          .attr('class', 'c-chart-gauge__needle')
          .attr('d', self._getPath(0.5));
        self = null;
    }.bind(this);

    this.update = function(p) {
        let self = this;
        self.group
            .select('path')
            .attr('d', self._getPath(p))
    }.bind(this);

    this._getPath = function(p) {
        let self = this;
        const thetaRad = percentToRadian(p / 2),
        centerX = 0,
        centerY = 0,
        topX = centerX - self.len * Math.cos(thetaRad),
        topY = centerY - self.len * Math.sin(thetaRad),
        leftX = centerX - self.radius * Math.cos(thetaRad - Math.PI / 2),
        leftY = centerY - self.radius * Math.sin(thetaRad - Math.PI / 2),
        rightX = centerX - self.radius * Math.cos(thetaRad + Math.PI / 2),
        rightY = centerY - self.radius * Math.sin(thetaRad + Math.PI / 2);
        self = null;
        return `M ${leftX} ${leftY} L ${topX} ${topY} L ${rightX} ${rightY}`;

    }.bind(this);
};

//TESTED
let GaugeChart = function(props) {

    this.svg = d3.select('#pressure-variation')
            .append("svg")
            .attr('class', 'c-chart-gauge')
            .attr("preserveAspectRatio", "xMinYMin meet")
            .attr("viewBox", "0 0 110 90")
            .classed("svg-content", true);

    this.group = this.svg.append('g');
    this.outerRadius = props.outerRadius;
    this.innerRadius = props.innerRadius;

    this.margin = {top: 11, right: 2, bottom: 2, left: 10};
    this.width = + this.svg.attr("width") -  this.margin.left -  this.margin.right;
    this.height = + this.svg.attr("height") -  this.margin.top -  this.margin.bottom;

    this.text =  this.svg.append('g');

    this.diffScale = undefined; 
    this.hPaText = undefined;

    this.needle = new Needle({
          svg: this.svg,
          len: this.outerRadius * 0.75,
          radius: this.innerRadius * 0.1,
          x: this.outerRadius + this.margin.left,
          y: this.outerRadius + this.margin.top
    })

    this.addText = function (x, y, _text, anchor, _class){
        let textElement = this.text.append("text");
        textElement
            .attr("class",_class)
            .attr("text-anchor", anchor)
            .attr("y", y)
            .attr("x", x )
            .text(_text);
        return textElement;
    }.bind(this);

    this.render = function() {
        let arc;
        let self = this;

        let sectionIndx = 0, _i = 0, arcStartRad, arcEndRad, startPadRad, endPadRad;
        let barWidth = 10;
        let numSections = 10;
        let sectionPerc = 1 / numSections / 2;
        let padRad = 0.05;
        let chartInset = 0;
        let totalPercent = .75;
        let radius = self.outerRadius;

        self.diffScale = d3.scaleLinear()
            .domain([-2, 2])
            .range([0, 1]);

        let x, y, thetaRad;
        for (sectionIndx = _i = 1; 1 <= numSections ? _i <= numSections : _i >= numSections; sectionIndx = 1 <= numSections ? ++_i : --_i) {
            arcStartRad = percentToRadian(totalPercent);
            arcEndRad = arcStartRad + percentToRadian(sectionPerc);
            totalPercent += sectionPerc;
            startPadRad = sectionIndx === 0 ? 0 : padRad / 2;
            endPadRad = sectionIndx === numSections ? 0 : padRad / 2;
            arc = d3.arc()
                .outerRadius(radius - chartInset)
                .innerRadius(radius - chartInset - barWidth)
                .startAngle(arcStartRad + startPadRad)
                .endAngle(arcEndRad - endPadRad);
            self.group
                .append('path')
                .attr('class', `chart-color${sectionIndx}`)
                .attr('d', arc)
                .attr("transform", `translate(${self.outerRadius + self.margin.left},${self.outerRadius + self.margin.top})`);

            switch(sectionIndx){
                case 1:
                    //case -2
                    thetaRad = percentToRadian(0 / 2);
                    x = -1;
                    y = self.outerRadius -1.75;
                    self.addText(x, y, "2", "end", "press-diff-text");
                    //case 2
                    thetaRad = percentToRadian(1 / 2);
                    x = self.outerRadius * 2 + 1;
                    self.addText(x, y, "2", "start", "press-diff-text");
                    break;
                case 3:
                    thetaRad = percentToRadian(0.25 / 2);
                    let r = self.outerRadius * 1.2;
                    //case -1
                    x = r - r * Math.cos(thetaRad) - 2;
                    y = r * Math.cos(thetaRad) - 23;
                    self.addText(x, y, "1", "end", "press-diff-text");
                    //case 1
                    x = r + r * Math.cos(thetaRad) - 17;
                    self.addText(x, y, "1", "start", "press-diff-text");
                    break;
                case 5:
                    thetaRad = percentToRadian(0.5);
                    //case 0
                    x = self.outerRadius;
                    y = 0;
                    self.addText(x, y, "0","middle", "press-diff-text");
                    break;
            }

        }
        x = self.outerRadius;
        y = self.outerRadius + 15;
        self.addText(x, y, "Tendência Pressão", "middle", "diff-press-first-line");
        y = y + 17;
        self.hPaText = self.addText(x, y, "0.0 hPa", "middle", "diff-press-second-line");
        self.hPaText.attr("id","hPaVelocityValue");
        self.text
            .attr("transform", `translate(${self.margin.left},${self.margin.top})`);
        self.needle.render();
        self = null;
        arc = null;
    }.bind(this);

    this.update = function(p) {
        let self = this;
        if (p < -2){
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(`temos uma velocidade da pressão fora dos 'limites' de : ${p}`));
            p = -2;
        }
        if (p > 2){
            log.addEntry(LOG_ENTRY_TYPE.ALERT, log.newEntry(`temos uma velocidade da pressão fora dos 'limites' de : ${p}`));
            p = 2;
        }
        self.needle.update(self.diffScale(p));
        if (p === undefined){
            self.hPaText.text("-.- hPa");
        }else{
            let val = p;
            if (val === -0.0){
                val = 0.0;
            }
            p = val;
            self.hPaText.text(`${val.toFixed(1)} hPa`);
        }
        self = null;
    }.bind(this);
};


let optionsTemp = {
    minutesWindow : minutesWindow,//120, //86400;
    chartId : "#temp-chart",
    // maxY : 50, //+ 47,3 °C	Amareleja	01/08/2003
    // minY : -16, //Penhas da Saúde	04/02/1954 // https://www.ipma.pt/pt/oclima/extremos.clima/index.jsp?page=extreme_co.xml
    maxY : 40, // real data analysis for aveiro, gives +32 as max since 2008 - have to check if gandara is similar.  Adjust if necessary
    minY : -5, // real data analysis for aveiro, gives -1,5 as min since 2008 - have to check if gandara is similar.  Adjust if necessary

    displayUnit : " ºC",
    roundDecimals : 1,
    xAxisYReference: 0
};

let optionsPressure = {
    minutesWindow : minutesWindow,//120, //86400;
    chartId : "#pressure-chart",
    // maxY : 1085,  //1084.5 - record value in Tosontsengel, Mongolia on 19 December 2001
    // minY : 870,  // 870, 12 October 1979, during Typhoon Tip in the western Pacific Ocean
    maxY : 1040,  // real data analysis for aveiro, gives 1035 as max since 2008 - have to check if gandara is similar.  Adjust if necessary
    minY : 910,  // real data analysis for aveiro, gives 910,6 as min since 2008 - have to check if gandara is similar.  Adjust if necessary

    displayUnit : " hpa",   
    roundDecimals : 0,
    xAxisYReference: 1013.25
};

let optionsWind = {
    minutesWindow : 180,//14400, //3 horas
    chartId : "#wind-chart",
    // maxWind : 180,  //177 is the portugues recordaccordingly with ipma site
    maxWind : 100,  //  real data analysis for aveiro, gives 57,6 as max since 2008 - have to check if gandara is similar.  Adjust if necessary
    minWind : 0,
    displayUnit : " km/h",
    roundDecimals : 0,
};

const optionsPressDiff = {
    outerRadius: 45, 
    innerRadius: 35 
};

let chartTemp = new Chart(optionsTemp);
let chartPress = new Chart(optionsPressure);
let chartWind = new Wind(optionsWind);
let gaugeChart = new GaugeChart(optionsPressDiff);
