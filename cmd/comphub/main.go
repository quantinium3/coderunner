package main

import (
	"fmt"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/quantinium03/comphub/internal/routes"
	"github.com/rs/cors"
)

func main() {
	r := chi.NewRouter()

	r.Use(cors.Default().Handler)
	r.Use(middleware.Logger)

	routes.SetupRoutes(r)

	fmt.Println("Server is running at port 3000, http://localhost:3000")
	http.ListenAndServe(":3000", r)
}
