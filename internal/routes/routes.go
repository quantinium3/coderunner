package routes

import (
	"github.com/go-chi/chi/v5"
	"github.com/quantinium03/comphub/internal/compilers"
)

func SetupRoutes(r chi.Router) {
	r.Post("/compile", compilers.CompileProgram)
}
