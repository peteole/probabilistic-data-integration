# Probabilistic Data Integration

This software allows integrating multiple probabilistic data sources into one unified view.

## Motivation

Consider a scenario where attributes should be gathered on entities from multiple data sources. For example, nutrition attributes such as the amount of fat and proteins in food products should be evaluated. The data sources could be for instance

1. The knowledge that every product can only contain between 0% and 100% of protein and fat
2. An estimate based on the category of the product
3. An estimate based on the ingredients list on the product
4. An accurate value known from a database such as OpenFoodFacts

All these pieces of information should be reconciled into one estimate for each quantity of interest.

## Theory

The most straightforward would be to always use the most accurate data source and ignore the rest. However, it is not always easy to say which data source this is: In the example of nutrition data, the estimate that the content of each quantity is between 0% and 100% may be very inaccurate most times compared to an accurate value from a database. But in some cases, the user may have made a mistake while converting units, resulting in an absurd value of fat in the database. Here, the very rough estimate should take over.

Therefore, a more advanced approach is chosen: each data source is modeled as a measurement with an uncertainty of some distribution. In other words, each data source $i$ states that the real value of the quantity is distributed as a random variable $Y_i$, with $g_i(x)=p(Y_i=x)$. The distribution of $Y_i$ depends on the real value of the variable that should be measured, $X$, and on the distribution of the error. Therefore, the likelihood for a value of $X$ stated by $g_i(x)\equiv p(Y_i=y_i|X=x)$. The values measured by the data sources are $y_i$. Then the following formula is used to compute the probability distribution of $x$:

$$
p(X=x) \propto \prod_{i}p(Y_i=y_i|X=x)=\prod_{i}g_i(x)
$$

The proportionality factor is set so that $p$ is a probability distribution.

There are multiple justifications for this way of reconciliation:

Assume the data sources to be independent. The first data source is used as a prior:

$$
p(X=x)=p(Y_1=y_1|X=x)
$$

In other words, it is assumed that a value of $X$ is just as likely as it is that the first data source measuring the value that has been measured given $X$.
Then the posterior of $X$ given that $Y_2=y_2$ can be computed using Bayes' theorem:

$$
p(X=x|Y_2=y_2)=\frac{p(Y_2=y_2|X=x)p(X=x)}{p(Y_2=y_2)}=\frac{g_1(x)g_2(x)}{\sum_{x'}p(Y_2=y_2|X=x')p(X=x')}=\frac{g_1(x)g_2(x)}{\sum_{x'}g_1(x')g_2(x')}
$$

Repeating this process for all data sources gives the result above.

## Features

Data sources:

- [x] GRPC
- [x] Rest
- [x] Mock
- [ ] SQL

Probability Distributions

- [x] Exact (delta)
- [x] Normal
- [x] Uniform
- [x] Discrete (strings)
- [ ] Tabular
- [ ] Mathematical expressions of probability distributions (sums, products...)

API

- [x] Rust function
- [x] GraphQL
- [x] REST

## Demo

The following exposes a GraphQL API that integrates several food data sources:

https://pdi.onrender.com/

You might try this query:

```graphql
query{
  search(query:"nutella"){
    energy_density{
      mu
      sigma
    }
    fat_density{
      mu
      sigma
      probability_density(x:[0.0,0.1,0.2,0.3,0.4,0.5,0.6])
    }
  }
}
```
And get the following result:
```json
{
  "data": {
    "search": {
      "energy_density": {
        "mu": 22519.999999999996,
        "sigma": 2221.7745400408608
      },
      "fat_density": {
        "mu": 0.30900000000000005,
        "sigma": 0.03048527233004555,
        "probability_density": [
          2.49690249539351e-21,
          1.5065849292388764e-9,
          0.025709552700972648,
          12.408071559301137,
          0.1693647107640426,
          6.538082957535243e-8,
          7.13816849862352e-19
        ]
      }
    }
  }
}
```

This demo uses [this](example_configs/food/config.yaml) config.