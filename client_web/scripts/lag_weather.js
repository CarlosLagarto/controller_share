"use strict";

var weather = function() {
    //TESTED
    this.render_chart = function(chart, current_minute, running_stat, ts, value){
        let data = chart.data;
        let l = data.length;

        if (ts > current_minute){
            running_stat.clear();
            running_stat.push(value);
            chart.render(ts, +running_stat.mean());
            current_minute = ts;
        }else{
            if (ts === current_minute){
                running_stat.push(value);
                if (l > 0){ 
                    data[l - 1] = { x: data[l - 1].x, y: +running_stat.mean()};
                }else{
                    // first point.  all other points goes to the above if part
                    data.push( {x: ts, y: running_stat.mean() } );
                }
            }else{
                // if client and backend time out of sync...
                let inx = binarySearch(data, ts);
                if (inx > -1){
                    data[inx].y = value;
                }else{
                    //do nothing - time not found
                }
            }
            chart.render();
        }
        data = null;
        return current_minute;
    }.bind(this);

    //TESTED
    this._clean_wind_value = function(wind_value){
        let value = Math.log10(wind_value);
        if (value < 0){
            value = 0;
        }
        return value;
    }.bind(this);

    //TESTED
    this.render_chart_wind = function(ts, wind, dir){
        let data = chartWind.data;
        let l = data.length;
        let value = 0;
        let self = this;

        if (ts <= self.current_minute_wind){
            self._running_stat_wind.push(wind);
            self._running_stat_dir.push(dir);
            value = self._clean_wind_value(+self._running_stat_wind.mean());
            if (value !== undefined){
                if (l > 0){ 
                    data[l - 1] = { ts: data[l - 1].ts, wind: value, dir: +self._running_stat_dir.mean()};
                }else{
                    // first point.  all other points goes to the above if part
                    data.push( {ts: ts, wind: value, dir: +self._running_stat_dir.mean() } );
                    self.current_minute_wind = ts
                }
            }
            // call with no parameters means refresh/render only
            chartWind.render();
        }else{
            self._running_stat_wind.clear();
            self._running_stat_dir.clear();
            self._running_stat_wind.push(wind);
            self._running_stat_dir.push(dir);
            value = self._clean_wind_value(+self._running_stat_wind.mean());
            chartWind.render(ts, value, +self._running_stat_dir.mean());
            self.current_minute_wind = ts;
        }
        self = null;
        data = null;
    }.bind(this);

    // TESTED
    this.render = function(weather_message){
        let payload = null;
        let aDate = null;
        let self = this;
        try{
            payload = weather_message.msg;
            show(self.wm.wind_needle);

            aDate = unix_to_js_date_adjusted(payload.current_time_ts);
            let direction = +payload.wind_bearing.toFixed(0);
            let intensity = +payload.wind_intensity.toFixed(0);
           
            let temp = +payload.temperature.toFixed(0);
            let press = +payload.pressure.toFixed(1);
            
            let tsNow = js_date_to_unix(aDate);
            let ts = Math.floor(tsNow / 60);
            
            self.wm.wind_direction.innerHTML = direction + unit_direction;
            self.wm.wind_intensity.innerHTML = intensity + unit_velocity;
            self.wm.wind_needle.style.transform = `rotate(${direction - 18}deg)`;
            self.wm.rain_today.innerHTML = payload.rain_today.toFixed(1) + unit_mm;
            self.wm.rain_week_acc.innerHTML = payload.rain_week_acc.toFixed(1) + unit_mm;

            self.wm.temp_now.innerHTML = temp + unit_celsius;
            
            self.wm.rain_prob_today.innerHTML = payload.rain_probability.toFixed(0) + unit_percentage;
            self.wm.humidity.innerHTML = payload.humidity.toFixed(0) + unit_percentage;

            self.wm.et.innerHTML = payload.et.toFixed(1) + unit_mm;

            if(self.historyReceived){
                // keep minute stats.  only show average each minute
                self.current_minute_temp = self.render_chart(chartTemp, self.current_minute_temp, self._running_stat_temp, ts, +temp);
                self.current_minute_press = self.render_chart(chartPress, self.current_minute_press, self._running_stat_press, ts, +press);
                self.render_chart_wind(ts, intensity, direction);
            }
            gaugeChart.update(payload.pressure_velocity);

        } catch(err){
            log.error(err);
        } finally{
            payload = null;
            aDate = null;
            self = null;
        }
    }.bind(this);

    //TESTED
    this.render_chart_history = function(chart, aValues, current_minute, ts, start){
        let l = aValues.length;
        if (l > 0) {
            for(let i = 0; i < l; i +=1){
                chart.data.push({x: aValues[i][0], y: +aValues[i][1] });
            }
            current_minute = +aValues[l-1][0];  
            chart.render();
        }else{
            current_minute = ts;
        }
        return current_minute;
    }.bind(this);

    //TESTED
    this.weatherHist = function(message){
        let msg = message.msg;
        let self = this;

        if(msg.request_uuid === this.requestUUID){
            let aTempPress = msg.temp_and_hp;
            let aWind = msg.wind;
            let end1 = msg.end1;
            let end2 = msg.end2;


            let l = aTempPress.length;
            let xi;
            let i = 0;
            self.current_minute_temp = end1;  
            self.current_minute_press = end1; 
            if (l > 0) {
                for(i = 0; i < l; i +=1){
                    xi = end1 - aTempPress[i].diff; //[0];
                    chartTemp.data.push({x: xi, y: +aTempPress[i].val1});//[1] });
                    chartPress.data.push({x: xi, y: +aTempPress[i].val2});//[2] });
                }
                chartTemp.render();
                chartPress.render();
            }
            // Vento
            l = aWind.length;
            let windValue = 0;
            self.current_minute_wind = end2;
            if (l > 0) {
                for(i = 0; i < l; i += 1){
                    xi = end2 - aWind[i].diff;//[0];
                    if (+aWind[i].val1 > 0){
                        windValue = Math.log10(+aWind[i].val1);//[1]);
                        if (windValue < 0){
                            windValue = 0;
                        }
                    }else{
                        windValue = 0;
                    }
                    chartWind.data.push({ts: xi, wind: windValue, dir: +aWind[i].val2});//[2] 
                }
                chartWind.render();
            }

            self.historyReceived = true;
            msg = null;

        }// else ignore
        self = null;
    }.bind(this);

    //TESTED
    this.connected_event = function(param){
        let msg = {};
        msg["ts"] = get_unix_now_time_adjusted();
        let message = MSG.BuildMessage(MSG.T.CTS_GET_WEATHER_HIST, msg);
        this.requestUUID = message.uuid();
        MQTT.SendMessage(message);
        // remove listener.  only meaningfull when connecting for the first time
        CTRL_CONN.connected_event.unregisterObserver(this);
    }.bind(this);


    //definitions
    this.wm = {
        wind_direction  : document.getElementById("weather_wind_direction"),
        wind_intensity  : document.getElementById("weather_wind_intensity"),
        wind_needle     : document.querySelector('#needle'),
        rain_today      : document.getElementById("weather_rain_today"),
        rain_week_acc   : document.getElementById("weather_rain_week_acc"),
        temp_now        : document.getElementById("weather_temp_now"),
        rain_prob_today : document.getElementById("weather_rain_prob_today"),
        humidity        : document.getElementById("weather_humidity"),
        et              : document.getElementById("weather_et"),
    };

    this.historyReceived = false;
    this.current_minute_temp = undefined;
    this.current_minute_press = undefined;
    this.current_minute_wind = undefined;
    this._running_stat_temp = new running_stat();
    this._running_stat_press = new running_stat();
    this._running_stat_wind = new running_stat();
    this._running_stat_dir = new running_stat();
    this.requestUUID  = undefined;
        
    //actions
    hide(this.wm.wind_needle);
    gaugeChart.render();

    MQTT.onSTCWeatherHistArrived = this.weatherHist;
    CTRL_CONN.connected_event.registerObserver(this, this.connected_event);

};
