/*
Copyright (C) 2020  Paul Kremer

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "thumbcreator.h"

extern "C" 
{
#include "stl2thumbnail.h"
}


#include <QImage>
#include <QString>

Q_LOGGING_CATEGORY(LOG_STL_THUMBS, "STLModelThumbs")

extern "C"
{
    // Factory method
    Q_DECL_EXPORT ThumbCreator* new_creator()
    {
        return new StlThumbCreator();
    }
};

StlThumbCreator::StlThumbCreator()
{
}

struct PicContainer {
    PictureBuffer buffer;
};

void cleanup(void* data) {
    auto container = static_cast<PicContainer*>(data);
    s2t_free_picture_buffer(container->buffer);
    delete container;
}

bool StlThumbCreator::create(const QString& path, int width, int height, QImage& img)
{
    //qCDebug(LOG_STL_THUMBS) << "Creating thumbnail for " << path;

    // render
    const auto pic = s2t_render(path.toStdString().c_str(), width, height);
    
    // failed?
    if(!pic.data)
        return false;
    
    // save the buffer in a container that stays around till it gets cleaned up
    // once 'img' goes out of scope
    auto container = new PicContainer{ pic };

    // QImage owns the buffer and it has to stay valid throughout the life of the QImage
    img = QImage(pic.data, width, height, pic.stride, QImage::Format_RGBA8888, cleanup, container);

    return true;
}
