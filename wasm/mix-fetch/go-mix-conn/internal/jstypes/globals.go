// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

package jstypes

import (
	"fmt"
	"net/url"
	"syscall/js"
)

var (
	Error    = js.Global().Get("Error")
	Promise  = js.Global().Get("Promise")
	Reflect  = js.Global().Get("Reflect")
	Object   = js.Global().Get("Object")
	Response = js.Global().Get("Response")
	Request  = js.Global().Get("Request")
	Proxy    = js.Global().Get("Proxy")
	Headers  = js.Global().Get("Headers")
)

func Origin() string {
	// nodejs doesn't have origin
	location := js.Global().Get("location")
	if !location.IsUndefined() && !location.IsNull() {
		return location.Get("origin").String()
	} else {
		return ""
	}
}

func OriginUrl() *url.URL {
	originUrl, originErr := url.Parse(Origin())
	if originErr != nil {
		panic(fmt.Sprintf("could not obtain origin: %s", originErr))
	}
	return originUrl
}
