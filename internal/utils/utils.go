package utils

import (
	"log"
	"os"
	"path/filepath"

	"github.com/google/uuid"
)

type createFileRes struct {
	Success  bool
	Error    string
	Filename string
}

func CreateFile(code string, language string) createFileRes {
	// Ensure "execution_zone" directory exists
	executionZone := "execution_zone"
	if _, err := os.Stat(executionZone); os.IsNotExist(err) {
		err := os.Mkdir(executionZone, 0755)
		if err != nil {
			log.Println("Directory could not be created")
			return createFileRes{
				Success: false,
				Error:   "Directory could not be created",
			}
		}
	}

	// Generate unique filename and write file in "execution_zone"
	filename := uuid.NewString() + "." + language
	filePath := filepath.Join(executionZone, filename)

	err := os.WriteFile(filePath, []byte(code), 0664) // 0600 for secure file permissions
	if err != nil {
		log.Println("File could not be created:", err)
		return createFileRes{
			Success: false,
			Error:   "File could not be created",
		}
	}

	return createFileRes{
		Success:  true,
		Filename: filePath,
	}
}

func DeleteFile(fileName string) {
	_, err := os.Stat(fileName)
	if os.IsNotExist(err) {
		log.Println("File or directory does not exist, skipping deletion:", fileName)
		return
	}

	err = os.Remove(fileName)
	if err != nil {
		log.Println("Failed to delete file ", fileName, "Error:", err)
	} else {
		log.Println("Successfully deleted file or directory:", fileName)
	}
}
