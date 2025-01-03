package handlers

import (
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/quantinium03/comphub/internal/compilers"
)

func SetupRoutes(r chi.Router) {
	r.Get("/compile", compilers.CompileProgram)
}
