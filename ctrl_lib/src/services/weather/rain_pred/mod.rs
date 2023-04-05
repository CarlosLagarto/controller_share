


// O tema da predição está a revelar-se mais complexo na implementação do que antecipei.
// implica (re)aprender alguma matemática, e implica pensar num modelo de forecast minimamente bom
// para isso é preciso dados para ensaiar soluções e construir o modelo
// Tenho duas ideias de base
// 1. utilizar a abordagem de https://www.researchgate.net/publication/337236701_Algorithm_to_Predict_the_Rainfall_Starting_Point_as_a_Function_of_Atmospheric_Pressure_Humidity_and_Dewpoint
//    e cruzar com a tendencia da pressão atmosferica nas ultimas 3 horas enquanto indicador do forecast
// 2. modelo de machine learning com as variaveis existentes para construir um modelo de forecast - naive bayes parece ser um bom candidato
// 
// E depois com isto contruir uma formula de probabilidade de chuva (o item 1 é o que dá em tese maior indicação de chuva em quantidade suficiente) para decidir se 
// esperamos / atrasamos 1 dia na rega para avaliar se poupamos água e luz como deve ser
//
// A ideia de base para o modelo naive bayes é construir um modelo enriquecendo os dados com features derivados dos dados base
// criar um conjunto de bins para a classificação da chuve, por exemplo, A=0, B=0..5, C=5..10, D=10..20, E=20..inf
// e com isto sempre que:
//      C com probabilidade > x% (80%) adiar 1 dia a rega para minimizar o gasto de agua
//      D com probabilidade > x% (80%) adiar 2 dia a rega para minimizar o gasto de agua
//      E com probabilidade > x% (80%) adiar 3 dia a rega para minimizar o gasto de agua e dar tempo para drenar o terreno
//
// A estratégia melhor parece entao ser terminar o programa na linha em que estava:
// - controlo de rega com automatico e wizard basico, 
// - controlo dos atuadores 
// - UI mobile
//
// e só depois atacar esta parte, senão nunca mais, e ainda tenho algum trabalho pela frente na parte do linux e dos phidgets
// por outro lado, este é o problema novo que dá mais pica :-)


pub mod naive_bayes;
pub mod data_structs;
