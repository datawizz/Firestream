import xmlrpc.client
import time

import pandas as pd

import os



class OdooClient:
    """
    The Odoo Client is a wrapper around the Odoo XMLRPC API.
    It is used to fetch data from Odoo and return it as a Pandas DataFrame.
    """

    def __init__(self, **kwargs):
        self.url = os.environ.get('ODOO_URL', kwargs.get('ODOO_URL'))
        self.db = os.environ.get('ODOO_DB_NAME', kwargs.get('ODOO_DB_NAME'))
        self.username = os.environ.get('ODOO_USERNAME', kwargs.get('ODOO_USERNAME'))
        self.password = os.environ.get('ODOO_PASSWORD', kwargs.get('ODOO_PASSWORD'))
        self.uid = self.authenticate()

    def authenticate(self):
        common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(self.url))
        return common.authenticate(self.db, self.username, self.password, {})


    def fetch_data_generator(self, model, domain, fields, limit=500):
        models = xmlrpc.client.ServerProxy(f'{self.url}/xmlrpc/2/object')
        
        def execute_with_backoff(*args, **kwargs):
            backoff = 0.5
            while True:
                try:
                    # print(f"Trying {args}")
                    return models.execute_kw(self.db, self.uid, self.password, *args, **kwargs)
                except Exception:
                    print(f"Backing Off {backoff}")
                    time.sleep(backoff)
                    backoff *= 2

        smallest_id = execute_with_backoff(model, 'search', [domain], {'order': 'id asc', 'limit': 1})[0]
        largest_id = execute_with_backoff(model, 'search', [domain], {'order': 'id desc', 'limit': 1})[0]
        print(f"In Table {model}")
        print(f"Lowest ID in Table: {smallest_id}")
        print(f"Highest ID In Table: {largest_id}")
        
        for current_id in range(smallest_id, largest_id + 1, limit):
            upper_bound = current_id + limit - 1
            search_ids = list(range(current_id, upper_bound + 1))
            data = execute_with_backoff(model, 'search_read', [domain + [('id', 'in', search_ids)]], {'fields': fields, 'order': 'id'})
            
            for record in data:
                yield record




    def process_json(self, data):
        for key, value in list(data.items()):
            if value is False:
                data[key] = None # Replace the Odoo default "False" with Pythonic "None"
            elif isinstance(value, list) and len(value) == 2:
                data[key] = value[0]
                data[key + '_name'] = value[1]
        return pd.DataFrame([data])


    def search_read(self, query: tuple):
        dfs = []
        for record in self.fetch_data_generator(*query):
            data = self.process_json(record)
            dfs.append(data)

        df = pd.concat(dfs)
        return df





