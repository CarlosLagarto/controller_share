<!-- Improved compatibility of back to top link: See: https://github.com/CarlosLagarto/controller_share -->
<a name="readme-top"></a>

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/CarlosLagarto/controller_share">
    <img src="images/logo.svg" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Lagarto Controller</h3>

  <!-- <p align="center">
    An awesome README template to jumpstart your projects!
    <br />
    <a href="https://github.com/othneildrew/Best-README-Template"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/othneildrew/Best-README-Template">View Demo</a>
    ·
    <a href="https://github.com/othneildrew/Best-README-Template/issues">Report Bug</a>
    ·
    <a href="https://github.com/othneildrew/Best-README-Template/issues">Request Feature</a>
  </p> -->
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <!-- <li><a href="#contributing">Contributing</a></li> -->
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project </p>
<p><strong>IoT backend controller + SPA Client + ML for next day rain probability.</strong></p>
<br>
<p><strong>Why:</strong> Just to keep my "nerd" spirit updated technology wise regarding HTML5, CSS3, SVG, RUST, Javascript ECMA2015, WebApps, Apache, SqLite, Linux, ...</p>
<br>
<p>First version was made with Python and MySql in the backend, but I simplified the database engine (sqlite uses less server resources than mysql) and with rust we use much less CPU, meaning lower power consuption (0%..0.5% versus 20%..40%)</p>
<p>Smart control for lawn watering, house central heater, integration and control of weather station data, and other sensors and actuators, with custom automation scenaries.</p>
<br>
<p><strong> How it work:</strong> Async sistributed system, with rust near real time controller, running on a intel NUC with linux/ubuntu.</p> 
<br>
<p>The controller integrates and bridges the external devices:</p>
<p>- <i>Shellies</i> to control the lawn water valves, central heater, shutters, esternal gate, etc.</p>
<p>- <i>Tempest</i> - weather station with readings each minute. Station transmit UDP packets, and we have a fail safe REST API channel to tempest site.</p>
<p>- <i>Mosquitto MQTT Broker</i> near real-time clients integration (commands and status reading).</p>
<p>- <i>Apache Web</i> for the SPA web application </p>
<p>- <i>Green IT</i> - power consuption bellow 0.5% CPU.</p>
<br>

<!-- [![Product Name Screen Shot][product-screenshot]](https://example.com) -->

<div align="center">
  <a href="https://github.com/CarlosLagarto/controller_share">
    <!-- <img src="images/logo.svg" alt="Logo" width="80" height="80">-->
    <img src="images/Screenshot_Weather.jpg" alt="Weather" width="50" height="80" > 
  </a>
  <a href="https://github.com/CarlosLagarto/controller_share">
    <!-- <img src="images/logo.svg" alt="Logo" width="80" height="80">-->
    <img src="images/Screenshot_LawnWatering.jpg" alt="Lawn" > 
  </a>
  <a href="https://github.com/CarlosLagarto/controller_share">
    <!-- <img src="images/logo.svg" alt="Logo" width="80" height="80">-->
    <img src="images/Screenshot_CENAS.jpg" alt="Devices" > 
  </a>
  <a href="https://github.com/CarlosLagarto/controller_share">
    <!-- <img src="images/logo.svg" alt="Logo" width="80" height="80">-->
    <img src="images/Screenshot_COISAS_Config.jpg" alt="Config" > 
  </a>  
</div>

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

* [![RUST][rust]][rust-url]
* [![Javacript][javascript]][javascript-url]
* [![Mosquitto][mosquitto]][mosquitto-url]
* [![Apache][apache]][apache-url]
* [![Shelly][shelly]][shelly-url]
* [![Tempest][tempest]][tempest-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

This is an example of how you may give instructions on setting up your project locally.
To get a local copy up and running follow these simple example steps.

### Prerequisites

This is an example of how to list things you need to use the software and how to install them.
* npm
  ```sh
  npm install npm@latest -g
  ```

### Installation

_Below is an example of how you can instruct your audience on installing and setting up your app. This template doesn't rely on any external dependencies or services._

1. Get a free API Key at [https://example.com](https://example.com)
2. Clone the repo
   ```sh
   git clone https://github.com/your_username_/Project-Name.git
   ```
3. Install NPM packages
   ```sh
   npm install
   ```
4. Enter your API in `config.js`
   ```js
   const API_KEY = 'ENTER YOUR API';
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- ROADMAP -->
## Roadmap

- [x] Add Changelog
- [x] Add back to top links
- [ ] Add Additional Templates w/ Examples
- [ ] Add "components" document to easily copy & paste sections of the readme
- [ ] Multi-language Support
    - [ ] Chinese
    - [ ] Spanish

See the [open issues](https://github.com/othneildrew/Best-README-Template/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Your Name - Carlos Lagarto

Project Link: [https://github.com/your_username/repo_name](https://github.com/your_username/repo_name)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

Just a special thanks to the RUST community and Mozzila that have made this journey easier and fun!

<p align="right">(<a href="#readme-top">back to top</a>)</p>

[rust]: https://img.shields.io/badge/Rust-1.68.2-yellowgreen?&style=for-the-badge
[rust-url]: https://www.rust-lang.org/
[javascript]: https://img.shields.io/badge/Javascript-ECMA2015-yellowgreen?&style=for-the-badge
[javascript-url]:https://developer.mozilla.org/pt-BR/docs/Web/JavaScript
[mosquitto]:https://img.shields.io/badge/mosquitto-MQTT%20Broker-yellowgreen?&style=for-the-badge
[mosquitto-url]:https://mosquitto.org/
[apache]: https://img.shields.io/badge/Apache-HTTP%20Server-yellowgreen
[apache-url]:https://httpd.apache.org/
[shelly]:https://img.shields.io/badge/Shelly-Home%20Automation-yellowgreen
[shelly-url]:https://www.shelly.cloud/en-pt
[tempest]:https://img.shields.io/badge/Tempest-Weather%20Station-yellowgreen
[tempest-url]: https://weatherflow.com/tempest-weather-system/