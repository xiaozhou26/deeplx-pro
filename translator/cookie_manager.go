package translator

import (
	"fmt"
	"log"
	"os"
	"strings"
)

var cookies []string
var invalidCookies []string
var currentCookieIndex int

func validateCookies() {
	cookieStr := os.Getenv("DEEPL_COOKIES")
	if cookieStr == "" {
		log.Fatal("No cookies provided. Please check your DEEPL_COOKIES environment variable.")
	}
	cookieList := strings.Split(cookieStr, ",")
	for _, cookie := range cookieList {
		cookie = strings.TrimSpace(cookie)
		if len(cookie) == 36 {
			cookies = append(cookies, cookie)
		} else {
			log.Printf("Invalid cookie format: %s", cookie)
		}
	}
	if len(cookies) == 0 {
		log.Fatal("No valid cookies provided. Please check your DEEPL_COOKIES environment variable.")
	}
}

func getNextCookie() (string, error) {
	attempts := 0
	for attempts < len(cookies) {
		cookie := cookies[currentCookieIndex]
		if !stringSliceContains(invalidCookies, cookie) {
			currentCookieIndex = (currentCookieIndex + 1) % len(cookies)
			return cookie, nil
		}
		currentCookieIndex = (currentCookieIndex + 1) % len(cookies)
		attempts++
	}
	return "", fmt.Errorf("no valid cookies available")
}

func markCookieInvalid(cookie string) {
	if !stringSliceContains(invalidCookies, cookie) {
		invalidCookies = append(invalidCookies, cookie)
	}
}

func initCookies() {
	validateCookies()
}
