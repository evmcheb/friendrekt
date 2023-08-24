import json
from flask import Flask, jsonify
from twitter.scraper import Scraper

with open("cookies.json") as f:
    cookies = json.load(f)

new = {}
for cookie in cookies:
    new[cookie["name"]] = cookie["value"]

scraper = Scraper(cookies=new)

# Initialize Flask app
app = Flask(__name__)

# Define the endpoint to get followers
@app.route('/followers/<int:userid>')
def get_followers(userid):
    user = scraper.users_by_ids([userid])
    print(user)
    return str(user[0]['data']['users'][0]['result']['legacy']['followers_count'])

if __name__ == '__main__':
    app.run(debug=True)