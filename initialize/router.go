package initialize

import (
	"deeplx-pro/translator"
	"math/rand"
	"net/http"

	"github.com/gin-gonic/gin"
)

// InitRouter initializes the Gin router with all necessary routes and middleware
func InitRouter() *gin.Engine {
	r := gin.Default()

	// 根路由，返回欢迎信息
	r.GET("/", func(c *gin.Context) {
		c.String(http.StatusOK, "Welcome to deeplx-pro")
	})

	// GET方法不支持翻译请求
	r.GET("/translate", func(c *gin.Context) {
		c.String(http.StatusMethodNotAllowed, "GET method not supported for this endpoint. Please use POST.")
	})

	// POST方法处理翻译请求
	r.POST("/translate", func(c *gin.Context) {
		var reqBody struct {
			Text       string `json:"text"`
			SourceLang string `json:"source_lang"`
			TargetLang string `json:"target_lang"`
			Quality    string `json:"quality"`
		}

		// 绑定JSON请求体
		if err := c.ShouldBindJSON(&reqBody); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request body", "details": err.Error()})
			return
		}
		// 检查源语言是否为"auto"
		if reqBody.SourceLang == "auto" || reqBody.SourceLang == "AUTO" {
			reqBody.SourceLang = "EN"
		}
		// 如果Quality为空，设置为默认值 "normal"
		if reqBody.Quality == "" {
			reqBody.Quality = "normal"
		}

		// 调用翻译函数
		result, err := translator.Translate(reqBody.Text, reqBody.SourceLang, reqBody.TargetLang, reqBody.Quality, 0)
		if err != nil || result == "" {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "Translation failed", "details": err.Error()})
			return
		}

		// 返回翻译结果
		response := gin.H{
			"alternatives": []string{},
			"code":         200,
			"data":         result,
			"id":           rand.Intn(10000000000),
			"source_lang":  reqBody.SourceLang,
			"target_lang":  reqBody.TargetLang,
			"quality":      reqBody.Quality,
		}
		c.JSON(http.StatusOK, response)
	})

	return r
}
