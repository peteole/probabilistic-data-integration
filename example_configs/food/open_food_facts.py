import openfoodfacts

from flask import Flask, json, request

app = Flask(__name__)
# for offline demo
cache = {
    'apple': {
        'numeric_fields': {
            'energy_density': {
                'Normal': {
                    'mu': 2500,
                    'sigma': 500
                }
            },
            'fat_density': {
                'Normal': {
                    'mu': 10,
                    'sigma': 3
                }
            },
        },
        'string_fields': {}
    }}


@app.route('/search', methods=['GET'])
def search():
    query = request.args.get('query')
    if query in cache.keys():
        return json.dumps(cache[query])
    product = openfoodfacts.products.search(query, page_size=2)['products'][0]
    print(product)
    nutriments = product['nutriments']
    print(nutriments)
    return json.dumps({
        'numeric_fields': {
            'energy_density': {
                'Normal': {
                    'mu': nutriments['energy_100g']/100*1000,
                    'sigma': 0.1*nutriments['energy_100g']/100*1000
                }
            },
            'fat_density': {
                'Normal': {
                    'mu': nutriments['fat_100g']/100,
                    'sigma': 0.1*nutriments['fat_100g']/100
                }
            },
        },
        'string_fields': {}
    })


if __name__ == '__main__':
    app.run()
