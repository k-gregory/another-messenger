#include <QApplication>
#include <QInputDialog>

#include "dialerwindow.h"

int main(int argc, char* argv[]){
    QApplication app(argc, argv);

    bool portOk = true;
    auto port = static_cast<quint16>(QInputDialog::getInt(
                                         nullptr,
                                         "Select port", "Choose port to listen", 4567, 0, 10000,1,&portOk));
    if(!portOk) return app.exec();

    DialerWindow wnd(port);
    wnd.show();

    return app.exec();
}
