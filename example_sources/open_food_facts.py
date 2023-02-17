import openfoodfacts

# result=openfoodfacts.products.search("apple",page_size=2)
# for r in result['products']:
#     nutriments=r['nutriments']
#     print()

# print(result)
from flask import Flask, json, request

companies = [{"id": 1, "name": "Company One"},
             {"id": 2, "name": "Company Two"}]

api = Flask(__name__)


@api.route('/search', methods=['GET'])
def search():
    query = request.args.get('query')
    product = openfoodfacts.products.search(query, page_size=2)['products'][0]
    nutriments = product['nutriments']
    print(nutriments)
    return json.dumps({
        'numeric_fields': {
            'energy': {
                'Normal': {
                    'mean': nutriments['energy'],
                    'sigma': 0.1*nutriments['energy']
                }
            },
            'fat': {
                'Normal': {
                    'mean': nutriments['fat'],
                    'sigma': 0.1*nutriments['fat']
                }
            },
        }
    })


if __name__ == '__main__':
    api.run()
