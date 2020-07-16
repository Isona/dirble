#!/usr/bin/env python3

from flask import Flask, url_for, redirect, abort, Response

# This needs to be global so that the decorators work
app = Flask(__name__)

@app.route("/")
def index():
	return "Index page"

@app.route("/302.html")
def code302():
	return redirect(url_for("index"), 302)

@app.route("/401.html")
def code401():
	abort(401)

@app.route("/403.html")
def code403():
	abort(403)

@app.route("/429.html")
def code429():
	abort(429)

def main():
	app.run(debug=True, host="::1", port=5000, threaded=True)

if __name__ == "__main__":
	main()