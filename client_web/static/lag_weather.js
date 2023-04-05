"use strict";

var weather = function() {

    //definições
    this.wm = {
        wind_direction  : document.getElementById("weather_wind_direction"),
        wind_intensity  : document.getElementById("weather_wind_intensity"),
        wind_needle     : document.querySelector('#needle'),
        rain_yesterday  : document.getElementById("weather_rain_yesterday"),
        rain_week_acc   : document.getElementById("weather_rain_week_acc"),
        temp_now        : document.getElementById("weather_temp_now"),
        rain_prob_today : document.getElementById("weather_rain_prob_today"),
        humidity        : document.getElementById("weather_humidity"),
        et              : document.getElementById("weather_et"),
        waiting_msg     : document.getElementById("weather_wait_msg")
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
    //let message = undefined;

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
                    //este é o primeiro ponto.  Daqui para a frente deve entrar na parte de cima do if
                    data.push( {x: ts, y: running_stat.mean() } );
                }
                // não passamos parametros, quer dizer que é só para fazer refresh com os dados que estão no array
                // porque foram atualizados aqui.
            }else{
                //o caso em que á desincronização entre o tempo do simulador e o tempo no cliente, ou simplesmente desacerto de relógios
                let inx = binarySearch(data, ts);
                if (inx > -1){
                    data[inx].y = value;
                }else{
                    //do nothing - o tempo não foi encontrado
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
                    //este é o primeiro ponto.  Daqui para a frente deve entrar na parte de cima do if
                    data.push( {ts: ts, wind: value, dir: +self._running_stat_dir.mean() } );
                    self.current_minute_wind = ts
                }
            }
            // não passamos parametros, quer dizer que é só para fazer refresh com os dados que estão no array
            // porque foram atualizados aqui.
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
            payload = weather_message.body.message;
            show(self.wm.wind_needle);

            aDate = payload.date;
            let direction = +payload.wind_direction.toFixed(0);
            let intensity = +payload.wind_intensity.toFixed(0);
            // noinspection JSUnresolvedVariable
            let temp = +payload.temperature_now.toFixed(0);
            let press = +payload.pressure.toFixed(1);
            let tsNow = moment(aDate, DATE_LONG_FORMAT).unix();
            let ts = Math.floor(tsNow / 60);
            
            self.wm.wind_direction.innerHTML = direction + unit_direction;
            self.wm.wind_intensity.innerHTML = intensity + unit_velocity;
            self.wm.wind_needle.style.transform = `rotate(${direction - 18}deg)`;
            self.wm.rain_yesterday.innerHTML = payload.rain_yesterday.toFixed(1) + unit_mm;
            self.wm.rain_week_acc.innerHTML = payload.weekly_acc_precipitation.toFixed(1) + unit_mm;

            self.wm.temp_now.innerHTML = temp + unit_celsius;
            // noinspection JSUnresolvedVariable
            self.wm.rain_prob_today.innerHTML = payload.rain_probability.toFixed(0) + unit_percentage;
            self.wm.humidity.innerHTML = payload.humidity.toFixed(0) + unit_percentage;

            // noinspection JSUnresolvedVariable
            self.wm.et.innerHTML = payload.et_value.toFixed(1) + unit_mm;

            if(self.historyReceived){
                //manter a estatistica a correr por cada minuto e mostrar apenas a média a cada minuto
                self.current_minute_temp = self.render_chart(chartTemp, self.current_minute_temp, self._running_stat_temp, ts, +temp);
                self.current_minute_press = self.render_chart(chartPress, self.current_minute_press, self._running_stat_press, ts, +press);
                self.render_chart_wind(ts, intensity, direction);
            }
            // noinspection JSUnresolvedVariable
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
    // noinspection JSUnusedGlobalSymbols
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
        let msg = message.get_message();
        let self = this;
        // noinspection JSUnresolvedVariable
        if(msg.request_uuid === this.requestUUID){
            removeClass(self.wm.waiting_msg.classList, "wait_status");
            hide(self.wm.waiting_msg); 

            // noinspection JSUnresolvedVariable
            let aTempPress = msg.temp_and_press;
            let aWind = msg.wind;
            // noinspection JSUnresolvedVariable
            let end1 = msg.end1;
            // noinspection JSUnresolvedVariable
            let end2 = msg.end2;


            let l = aTempPress.length;
            let xi;
            let i = 0;
            self.current_minute_temp = end1;  
            self.current_minute_press = end1; 
            if (l > 0) {
                for(i = 0; i < l; i +=1){
                    xi = end1 - aTempPress[i][0];
                    chartTemp.data.push({x: xi, y: +aTempPress[i][1] });
                    chartPress.data.push({x: xi, y: +aTempPress[i][2] });
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
                    xi = end2 - aWind[i][0];
                    if (+aWind[i][1] > 0){
                        windValue = Math.log10(+aWind[i][1]);
                        if (windValue < 0){
                            windValue = 0;
                        }
                    }else{
                        windValue = 0;
                    }
                    chartWind.data.push({ts: xi, wind: windValue, dir: +aWind[i][2] });
                }
                chartWind.render();
            }

            self.historyReceived = true;
            msg = null;

        }// else ignoramos porque não foi este cliente a pedir.
    }.bind(this);

    //TESTED
    this.notifyConnection = function(status){
        if (status){
            let msg = {};
            msg["ts"] = moment().unix();
            let message = MSG.BuildMessage(MSG.T.CTS_GET_WEATHER_HIST, msg);
            this.requestUUID = message.uuid();
            MQTT.SendMessage(message);
            //retiramos o listener.  Só da primeira vez é que quero ver o gráfico.  
            // das vezes seguintes, ou a pessoa faz refresh, ou a interrupção é curta - não vamos estar sempre a olhar para o gráfico
            MQTT.connectionEvent.unregisterObserver(this);
        } 
    }.bind(this);

    //ações
    hide(this.wm.wind_needle);
    gaugeChart.render();

    MQTT.onSTCWeatherHistArrived = this.weatherHist;
    MQTT.connectionEvent.registerObserver(this);

};
