# A flask app that serves a endpoint for parsing a URL

from flask import Flask, request, jsonify
import main

app = Flask(__name__)

@app.route('/parse', methods=['POST'])
def parse_url():
    data = request.json
    url = data.get('url', '')

    if not url:
        return jsonify({'error': 'URL not provided'}), 400

    result = main.parse_url(url)

    return jsonify({'result': result})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
